//! PAINT 后端库入口：模块声明、应用状态、Tauri 命令注册。
//!
//! 所有命令都定义在 [`commands`] 子模块里：`#[tauri::command]` 会 `#[macro_export]` 一个
//! `__cmd__<name>` 宏，定义在 crate 根时会与 `pub use` 撞在同一个宏命名空间（E0255）。
//!
//! 命令与事件契约见 `docs/engineering-contract.md` 第 6 节。

pub mod ai;
pub mod employees;
pub mod error;
pub mod events;
pub mod fsutil;
pub mod harness;
pub mod harness_web;
pub mod knowledge;
pub mod paint;
pub mod paths;
pub mod skills;
pub mod store;

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::Manager;

use ai::engines::{DirectEngine, HarnessEngine};
use ai::gateway::{ChatEngine, Gateway};
use error::AppError;
use events::{AppHandleSink, EventSink};
use harness::{HarnessPaths, SandboxPolicy};
use harness_web::WebUi;
use paths::PaintHome;
use store::Store;

pub use error::ErrorKind;

/// `ping` 命令的返回体，与 `src/lib/types.ts` 的 `PingResponse` 对应。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    pub pong: bool,
    pub version: String,
}

/// 构造 ping 响应的纯函数（便于单测，不依赖 Tauri 运行时）。
pub fn ping_response() -> PingResponse {
    PingResponse {
        pong: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// 进程级应用状态。
pub struct AppState {
    pub home: PaintHome,
    pub store: Arc<Mutex<Store>>,
    pub gateway: Arc<Gateway>,
    pub web: Arc<WebUi>,
    pub sink: Arc<dyn EventSink>,
}

impl AppState {
    /// 加锁访问数据库（所有命令统一走这里）。
    pub fn with_store<T>(
        &self,
        action: impl FnOnce(&Store) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let guard = self
            .store
            .lock()
            .map_err(|_| AppError::db("数据库连接已被污染，请重启 PAINT"))?;
        action(&guard)
    }
}

/// 根据本机环境组装 AI 网关：DSH 可用 + 有 API Key 才装 harness。
pub fn build_gateway(home: &PaintHome) -> Gateway {
    let harness_engine = HarnessPaths::resolve(home)
        .ok()
        .filter(|paths| paths.is_available())
        .and_then(|paths| {
            let api_key = std::env::var(ai::engines::ENV_API_KEY).unwrap_or_default();
            if api_key.trim().is_empty() {
                return None;
            }
            let policy = std::env::var(harness::ENV_SANDBOX)
                .ok()
                .and_then(|value| SandboxPolicy::parse(&value).ok())
                .unwrap_or(SandboxPolicy::CanvasWrite);
            Some(Arc::new(HarnessEngine::new(
                paths,
                home.root(),
                policy,
                api_key,
            )) as Arc<dyn ChatEngine>)
        });

    let direct_engine =
        DirectEngine::from_env().map(|engine| Arc::new(engine) as Arc<dyn ChatEngine>);

    Gateway::new(harness_engine, direct_engine)
}

/// Tauri 命令集合。
pub mod commands {
    use std::path::PathBuf;

    use serde_json::json;
    use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};

    use super::*;
    use crate::ai::gateway::{self as ai_gateway, AiCall, AiChatRequest, AiEngine, AiUsageRow};
    use crate::ai::ops::{self, OpsPolicy, OpsReport};
    use crate::employees::{self, Employee, EmployeeInput};
    use crate::events::{
        EVENT_AI_ERROR, EVENT_CANVAS_CHANGED, EVENT_DSH_WEB_STATUS,
    };
    use crate::harness;
    use crate::harness_web::{self, DshWebInfo};
    use crate::knowledge::{self, KnowledgeEntry, WorkspaceRegistration};
    use crate::paint::command_dispatcher;
    use crate::paint::document::{Actor, CanvasDocument, CanvasSummary};
    use crate::paint::files::{self, ExportFormat};
    use crate::paint::service::{self, HistoryStatus};
    use crate::paint::state_machine::ActionRequest;
    use crate::skills::{self, SkillInfo, SyncReport};

    /// 探活。
    #[tauri::command]
    pub fn ping() -> PingResponse {
        crate::ping_response()
    }

    // ------------------------------------------------------------------
    // 画布
    // ------------------------------------------------------------------

    #[tauri::command]
    pub fn create_canvas(
        state: State<'_, AppState>,
        name: String,
        width: u32,
        height: u32,
    ) -> Result<CanvasDocument, AppError> {
        state.with_store(|store| {
            command_dispatcher::create_canvas(store, state.sink.as_ref(), name, width, height)
        })
    }

    #[tauri::command]
    pub fn get_canvas(
        state: State<'_, AppState>,
        canvas_id: String,
    ) -> Result<CanvasDocument, AppError> {
        state.with_store(|store| service::load_canvas(store, &canvas_id))
    }

    #[tauri::command]
    pub fn list_canvases(state: State<'_, AppState>) -> Result<Vec<CanvasSummary>, AppError> {
        state.with_store(service::list_canvases)
    }

    #[tauri::command]
    pub fn apply_action(
        state: State<'_, AppState>,
        canvas_id: String,
        action: String,
        input: serde_json::Value,
        actor: Actor,
        expected_revision: Option<u64>,
    ) -> Result<CanvasDocument, AppError> {
        let request = ActionRequest {
            action,
            input,
            actor,
            expected_revision,
        };
        state.with_store(|store| {
            command_dispatcher::dispatch(store, state.sink.as_ref(), &canvas_id, request)
        })
    }

    #[tauri::command]
    pub fn undo(state: State<'_, AppState>, canvas_id: String) -> Result<CanvasDocument, AppError> {
        state.with_store(|store| {
            command_dispatcher::dispatch(
                store,
                state.sink.as_ref(),
                &canvas_id,
                ActionRequest::new("undo", json!({}), Actor::User),
            )
        })
    }

    #[tauri::command]
    pub fn redo(state: State<'_, AppState>, canvas_id: String) -> Result<CanvasDocument, AppError> {
        state.with_store(|store| {
            command_dispatcher::dispatch(
                store,
                state.sink.as_ref(),
                &canvas_id,
                ActionRequest::new("redo", json!({}), Actor::User),
            )
        })
    }

    #[tauri::command]
    pub fn history_status(
        state: State<'_, AppState>,
        canvas_id: String,
    ) -> Result<HistoryStatus, AppError> {
        state.with_store(|store| command_dispatcher::history_status(store, &canvas_id))
    }

    // ------------------------------------------------------------------
    // 文件
    // ------------------------------------------------------------------

    #[tauri::command]
    pub async fn pick_save_path(default_name: String) -> Result<Option<String>, AppError> {
        tauri::async_runtime::spawn_blocking(move || {
            rfd::FileDialog::new()
                .set_file_name(&default_name)
                .add_filter("PAINT 作品", &[files::PAINT_FILE_EXTENSION])
                .save_file()
                .map(|path| path.to_string_lossy().into_owned())
        })
        .await
        .map_err(|error| AppError::io(format!("对话框线程失败：{error}")))
    }

    #[tauri::command]
    pub async fn pick_open_path() -> Result<Option<String>, AppError> {
        tauri::async_runtime::spawn_blocking(move || {
            rfd::FileDialog::new()
                .add_filter("PAINT 作品", &[files::PAINT_FILE_EXTENSION])
                .pick_file()
                .map(|path| path.to_string_lossy().into_owned())
        })
        .await
        .map_err(|error| AppError::io(format!("对话框线程失败：{error}")))
    }

    #[tauri::command]
    pub async fn pick_export_path(default_name: String) -> Result<Option<String>, AppError> {
        tauri::async_runtime::spawn_blocking(move || {
            rfd::FileDialog::new()
                .set_file_name(&default_name)
                .add_filter("PNG", &["png"])
                .add_filter("JPG", &["jpg", "jpeg"])
                .add_filter("SVG", &["svg"])
                .save_file()
                .map(|path| path.to_string_lossy().into_owned())
        })
        .await
        .map_err(|error| AppError::io(format!("对话框线程失败：{error}")))
    }

    #[tauri::command]
    pub fn save_document(
        state: State<'_, AppState>,
        canvas_id: String,
        path: String,
    ) -> Result<String, AppError> {
        state.with_store(|store| {
            files::save_paint_file(store, &canvas_id, PathBuf::from(path))
                .map(|path| path.to_string_lossy().into_owned())
        })
    }

    #[tauri::command]
    pub fn open_document(
        state: State<'_, AppState>,
        path: String,
    ) -> Result<CanvasDocument, AppError> {
        state.with_store(|store| {
            let document = files::open_paint_file(store, PathBuf::from(path))?;
            let payload = serde_json::to_value(&document)?;
            state.sink.emit(EVENT_CANVAS_CHANGED, payload);
            Ok(document)
        })
    }

    #[tauri::command]
    pub fn export_image(
        path: String,
        format: String,
        data_url: String,
    ) -> Result<String, AppError> {
        let format = ExportFormat::parse(&format)?;
        files::export_image(PathBuf::from(path), format, &data_url)
            .map(|path| path.to_string_lossy().into_owned())
    }

    // ------------------------------------------------------------------
    // AI
    // ------------------------------------------------------------------

    #[tauri::command]
    pub fn ai_chat_start(
        state: State<'_, AppState>,
        request: AiChatRequest,
    ) -> Result<AiChatStart, AppError> {
        let engine = state.gateway.select_engine(&request)?;
        let session_id = ai_gateway::new_session_id();
        let gateway = state.gateway.clone();
        let sink = state.sink.clone();
        let store_path = state.home.db_path();

        let start = AiChatStart {
            session_id: session_id.clone(),
            engine,
            model: match engine {
                AiEngine::Harness => ai::engines::HARNESS_DEFAULT_MODEL.to_string(),
                AiEngine::Direct => ai_gateway::DEFAULT_DIRECT_MODEL.to_string(),
            },
        };

        std::thread::spawn(move || {
            let result = Store::open(&store_path).and_then(|store| {
                let summary = gateway.chat_with_session(
                    &store,
                    sink.as_ref(),
                    request.clone(),
                    session_id.clone(),
                )?;
                if summary.success {
                    if let Some(canvas_id) = request.canvas_id.as_deref() {
                        let report: OpsReport = ops::apply_from_text(
                            &store,
                            sink.as_ref(),
                            canvas_id,
                            &summary.text,
                            OpsPolicy::default(),
                        )?;
                        return Ok::<_, AppError>((summary, report));
                    }
                }
                Ok((summary, OpsReport::default()))
            });

            if let Err(error) = result {
                sink.emit(
                    EVENT_AI_ERROR,
                    json!({
                        "sessionId": session_id,
                        "kind": error.kind.as_str(),
                        "message": error.message,
                    }),
                );
            }
        });

        Ok(start)
    }

    #[tauri::command]
    pub fn ai_cancel() -> Result<(), AppError> {
        harness::request_cancel();
        Ok(())
    }

    #[tauri::command]
    pub fn ai_list_calls(state: State<'_, AppState>, limit: u32) -> Result<Vec<AiCall>, AppError> {
        state.with_store(|store| ai_gateway::list_calls(store, limit))
    }

    #[tauri::command]
    pub fn ai_usage_summary(
        state: State<'_, AppState>,
        days: u32,
    ) -> Result<Vec<AiUsageRow>, AppError> {
        state.with_store(|store| ai_gateway::usage_summary(store, days))
    }

    // ------------------------------------------------------------------
    // 数字员工
    // ------------------------------------------------------------------

    #[tauri::command]
    pub fn list_employees(state: State<'_, AppState>) -> Result<Vec<Employee>, AppError> {
        state.with_store(employees::list)
    }

    #[tauri::command]
    pub fn save_employee(
        state: State<'_, AppState>,
        employee: EmployeeInput,
    ) -> Result<Employee, AppError> {
        let saved = state.with_store(|store| employees::save(store, employee))?;
        state.with_store(|store| employees::sync_employees(&state.home, store))?;
        Ok(saved)
    }

    #[tauri::command]
    pub fn delete_employee(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
        state.with_store(|store| employees::delete(store, &id))?;
        state.with_store(|store| employees::sync_employees(&state.home, store))?;
        Ok(())
    }

    #[tauri::command]
    pub fn sync_employees(state: State<'_, AppState>) -> Result<Vec<String>, AppError> {
        state.with_store(|store| employees::sync_employees(&state.home, store))
    }

    // ------------------------------------------------------------------
    // 技能
    // ------------------------------------------------------------------

    #[tauri::command]
    pub fn list_skills(state: State<'_, AppState>) -> Result<Vec<SkillInfo>, AppError> {
        skills::list_skills(&state.home)
    }

    #[tauri::command]
    pub fn set_skill_enabled(
        state: State<'_, AppState>,
        id: String,
        enabled: bool,
    ) -> Result<(), AppError> {
        skills::set_skill_enabled(&state.home, &id, enabled)
    }

    #[tauri::command]
    pub fn sync_skills(state: State<'_, AppState>) -> Result<SyncReport, AppError> {
        skills::sync_skills(&state.home)
    }

    #[tauri::command]
    pub fn import_dsh_skill(state: State<'_, AppState>, id: String) -> Result<SkillInfo, AppError> {
        skills::import_dsh_skill(&state.home, &id)
    }

    // ------------------------------------------------------------------
    // 知识库
    // ------------------------------------------------------------------

    #[tauri::command]
    pub fn knowledge_dirs(state: State<'_, AppState>) -> Result<KnowledgeDirs, AppError> {
        knowledge::ensure_layout(&state.home)?;
        Ok(KnowledgeDirs {
            root: knowledge::knowledge_root(&state.home)
                .to_string_lossy()
                .into_owned(),
            dirs: knowledge::KNOWLEDGE_KINDS
                .iter()
                .map(|kind| kind.to_string())
                .collect(),
        })
    }

    #[tauri::command]
    pub fn list_knowledge(
        state: State<'_, AppState>,
        kind: String,
    ) -> Result<Vec<KnowledgeEntry>, AppError> {
        knowledge::list_entries(&state.home, &kind)
    }

    #[tauri::command]
    pub fn read_knowledge(
        state: State<'_, AppState>,
        kind: String,
        name: String,
    ) -> Result<KnowledgeEntry, AppError> {
        knowledge::read_entry(&state.home, &kind, &name)
    }

    #[tauri::command]
    pub fn save_knowledge_entry(
        state: State<'_, AppState>,
        kind: String,
        name: String,
        content: String,
    ) -> Result<KnowledgeEntry, AppError> {
        knowledge::save_entry(&state.home, &kind, &name, &content)
    }

    #[tauri::command]
    pub fn delete_knowledge_entry(
        state: State<'_, AppState>,
        kind: String,
        name: String,
    ) -> Result<(), AppError> {
        knowledge::delete_entry(&state.home, &kind, &name)
    }

    #[tauri::command]
    pub fn register_knowledge_workspace(
        state: State<'_, AppState>,
    ) -> Result<WorkspaceRegistration, AppError> {
        knowledge::register_workspace(&state.home)
    }

    // ------------------------------------------------------------------
    // DSH Web UI
    // ------------------------------------------------------------------

    #[tauri::command]
    pub async fn start_dsh_web(state: State<'_, AppState>) -> Result<DshWebInfo, AppError> {
        let paths = HarnessPaths::resolve(&state.home)?;
        if !paths.is_available() {
            return Err(AppError::harness(
                "未找到 DSH，请先安装 deepseek-desktop 或设置 PAINT_DSH_ENTRY",
            ));
        }
        let web = state.web.clone();
        let sink = state.sink.clone();

        let info = tauri::async_runtime::spawn_blocking(move || {
            web.start(
                &paths,
                harness_web::DEFAULT_HOST,
                harness_web::EPHEMERAL_PORT,
            )
        })
        .await
        .map_err(|error| AppError::harness(format!("启动线程失败：{error}")))??;

        sink.emit(
            EVENT_DSH_WEB_STATUS,
            json!({ "running": true, "url": info.url, "port": info.port }),
        );
        Ok(info)
    }

    #[tauri::command]
    pub fn stop_dsh_web(state: State<'_, AppState>) -> Result<(), AppError> {
        state.web.stop()?;
        state
            .sink
            .emit(EVENT_DSH_WEB_STATUS, json!({ "running": false }));
        Ok(())
    }

    #[tauri::command]
    pub fn dsh_web_status(state: State<'_, AppState>) -> Option<DshWebInfo> {
        state.web.status()
    }

    /// 用独立 WebviewWindow 打开 DSH Web UI。
    ///
    /// 不用 iframe：DSH 的鉴权 cookie 是 `SameSite=Strict`，iframe 属第三方上下文会全部 401。
    #[tauri::command]
    pub fn open_dsh_web_window(
        app: tauri::AppHandle,
        state: State<'_, AppState>,
    ) -> Result<(), AppError> {
        let info = state
            .web
            .status()
            .ok_or_else(|| AppError::harness("DSH Web UI 尚未启动"))?;
        let url: tauri::Url = info
            .url
            .parse()
            .map_err(|error| AppError::harness(format!("无法解析 DSH URL：{error}")))?;

        if let Some(window) = app.get_webview_window("dsh") {
            let _ = window.close();
        }

        WebviewWindowBuilder::new(&app, "dsh", WebviewUrl::External(url))
            .title("PAINT · AI 工作台")
            .inner_size(1120.0, 820.0)
            .initialization_script(harness_web::brand_script())
            .build()
            .map_err(|error| AppError::harness(format!("打开 DSH 窗口失败：{error}")))?;

        Ok(())
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AiChatStart {
        pub session_id: String,
        pub engine: AiEngine,
        pub model: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct KnowledgeDirs {
        pub root: String,
        pub dirs: Vec<String>,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let sink: Arc<dyn EventSink> = Arc::new(AppHandleSink::new(handle));

            let home = PaintHome::resolve()?;
            home.ensure_dirs()?;

            let store = Store::open(home.db_path())?;
            employees::seed_builtins(&store)?;
            skills::seed_builtin_skills(&home)?;
            knowledge::ensure_layout(&home)?;

            // 首次启动即把员工 preset / 技能 / 知识索引同步到 DSH_HOME（幂等）。
            let _ = employees::sync_employees(&home, &store);
            let _ = skills::sync_skills(&home);
            let _ = knowledge::sync_knowledge_agents(&home);

            let gateway = Arc::new(build_gateway(&home));

            app.manage(AppState {
                home,
                store: Arc::new(Mutex::new(store)),
                gateway,
                web: Arc::new(WebUi::new()),
                sink,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::create_canvas,
            commands::get_canvas,
            commands::list_canvases,
            commands::apply_action,
            commands::undo,
            commands::redo,
            commands::history_status,
            commands::pick_save_path,
            commands::pick_open_path,
            commands::pick_export_path,
            commands::save_document,
            commands::open_document,
            commands::export_image,
            commands::ai_chat_start,
            commands::ai_cancel,
            commands::ai_list_calls,
            commands::ai_usage_summary,
            commands::list_employees,
            commands::save_employee,
            commands::delete_employee,
            commands::sync_employees,
            commands::list_skills,
            commands::set_skill_enabled,
            commands::sync_skills,
            commands::import_dsh_skill,
            commands::knowledge_dirs,
            commands::list_knowledge,
            commands::read_knowledge,
            commands::save_knowledge_entry,
            commands::delete_knowledge_entry,
            commands::register_knowledge_workspace,
            commands::start_dsh_web,
            commands::stop_dsh_web,
            commands::dsh_web_status,
            commands::open_dsh_web_window,
        ])
        .run(tauri::generate_context!())
        .expect("PAINT 启动失败");
}

#[cfg(test)]
mod tests {
    // 注意：不要写 `use super::*;`——`#[tauri::command]` 会生成 `__cmd__*` 宏，
    // glob 导入会把它重复导入同一个模块，触发 E0255。
    use super::ping_response;

    #[test]
    fn ping_returns_pong_with_crate_version() {
        let response = ping_response();

        assert!(response.pong);
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
        assert!(!response.version.is_empty());
    }

    #[test]
    fn ping_response_serializes_with_camel_case_fields() {
        let value = serde_json::to_value(ping_response()).expect("序列化 PingResponse");

        assert_eq!(value["pong"], serde_json::json!(true));
        assert_eq!(value["version"], serde_json::json!(env!("CARGO_PKG_VERSION")));
    }
}
