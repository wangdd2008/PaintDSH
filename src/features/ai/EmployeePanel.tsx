import { useEffect, useState } from "react";

import * as bridge from "../../lib/bridge";
import type { Employee, EmployeeInput } from "../../lib/types";

const EMPTY_DRAFT: EmployeeInput = {
  name: "",
  role: "",
  persona: "",
  skills: ["canvas-create-layer"],
  knowledge: ["palettes"],
};

/** 数字员工面板：内置三类 + 自定义 CRUD + 同步 preset。 */
export function EmployeePanel() {
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [selected, setSelected] = useState<string>("");
  const [draft, setDraft] = useState<EmployeeInput>(EMPTY_DRAFT);
  const [status, setStatus] = useState<string | null>(null);

  const refresh = async () => {
    try {
      const list = await bridge.listEmployees();
      setEmployees(list);
      if (!selected && list.length > 0) {
        setSelected(list[0].id);
      }
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  useEffect(() => {
    void refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const current = employees.find((employee) => employee.id === selected) ?? null;

  useEffect(() => {
    if (current) {
      setDraft({
        id: current.id,
        name: current.name,
        role: current.role,
        persona: current.persona,
        skills: current.skills,
        knowledge: current.knowledge,
        model: current.model ?? null,
      });
    }
  }, [current?.id]); // eslint-disable-line react-hooks/exhaustive-deps

  const save = async () => {
    try {
      await bridge.saveEmployee(draft);
      setStatus("已保存并同步 preset");
      await refresh();
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  const remove = async () => {
    if (!draft.id) {
      return;
    }
    try {
      await bridge.deleteEmployee(draft.id);
      setSelected("");
      setDraft(EMPTY_DRAFT);
      setStatus("已删除");
      await refresh();
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <section className="employee-panel" data-testid="employee-panel" aria-label="数字员工">
      <header className="panel-section__header">
        <h2>数字员工</h2>
        <button type="button" onClick={() => void bridge.syncEmployees().then(() => setStatus("已同步"))}>
          同步
        </button>
      </header>

      <ul className="employee-list">
        {employees.map((employee) => (
          <li key={employee.id}>
            <button
              type="button"
              className={employee.id === selected ? "is-active" : undefined}
              onClick={() => setSelected(employee.id)}
            >
              {employee.name}
              {employee.builtin ? <span className="badge">内置</span> : null}
            </button>
          </li>
        ))}
      </ul>

      <div className="employee-form">
        <label>
          名称
          <input
            value={draft.name}
            onChange={(event) => setDraft({ ...draft, name: event.target.value })}
          />
        </label>
        <label>
          职责
          <input
            value={draft.role}
            onChange={(event) => setDraft({ ...draft, role: event.target.value })}
          />
        </label>
        <label>
          角色设定
          <textarea
            rows={4}
            value={draft.persona}
            onChange={(event) => setDraft({ ...draft, persona: event.target.value })}
          />
        </label>
        <label>
          技能（逗号分隔）
          <input
            value={draft.skills.join(", ")}
            onChange={(event) =>
              setDraft({
                ...draft,
                skills: event.target.value
                  .split(",")
                  .map((item) => item.trim())
                  .filter(Boolean),
              })
            }
          />
        </label>
        <label>
          知识库（逗号分隔）
          <input
            value={draft.knowledge.join(", ")}
            onChange={(event) =>
              setDraft({
                ...draft,
                knowledge: event.target.value
                  .split(",")
                  .map((item) => item.trim())
                  .filter(Boolean),
              })
            }
          />
        </label>
        <div className="employee-form__actions">
          <button type="button" onClick={() => void save()} disabled={!draft.name || !draft.persona}>
            {draft.id ? "保存" : "新建"}
          </button>
          <button type="button" onClick={() => setDraft(EMPTY_DRAFT)}>
            清空
          </button>
          <button type="button" onClick={() => void remove()} disabled={!draft.id || current?.builtin}>
            删除
          </button>
        </div>
        {status ? <p className="employee-form__status">{status}</p> : null}
      </div>
    </section>
  );
}
