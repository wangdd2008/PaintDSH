// EChart 通用封装
function EChart({ option, style, onReady }) {
  const ref = React.useRef(null);
  const chartRef = React.useRef(null);
  React.useEffect(() => {
    const el = ref.current;
    const chart = echarts.init(el, null, { renderer: 'canvas' });
    chartRef.current = chart;
    chart.setOption(option);
    const ro = new ResizeObserver(() => chart.resize());
    ro.observe(el);
    if (onReady) onReady(chart);
    return () => { ro.disconnect(); chart.dispose(); };
    // eslint-disable-next-line
  }, []);
  React.useEffect(() => {
    if (chartRef.current) {
      chartRef.current.setOption(option, true);
    }
  }, [option]);
  return <div ref={ref} style={{ width: '100%', height: '100%', minHeight: 120, ...style }} />;
}

// 生成时间序列数据（用于实时监控）
function generateTimeData(points = 40, base = 30, variance = 20, min = 0, max = 100) {
  const data = [];
  for (let i = 0; i < points; i++) {
    let v = base + (Math.random() - 0.5) * variance * 2 + Math.sin(i * 0.3) * variance * 0.3;
    v = Math.max(min, Math.min(max, v));
    data.push(+v.toFixed(1));
  }
  return data;
}

function generateXAxisTime(points = 40) {
  const labels = [];
  const now = Date.now();
  for (let i = points - 1; i >= 0; i--) {
    const t = new Date(now - i * 2000);
    labels.push(t.getMinutes().toString().padStart(2, '0') + ':' + t.getSeconds().toString().padStart(2, '0'));
  }
  return labels;
}

// ─── M1 系统监控主面板 ───────────────────────────────────────
function SystemMonitor({ compact = false }) {
  const [data, setData] = React.useState(() => ({
    pCore: generateTimeData(40, 45, 15),
    eCore: generateTimeData(40, 25, 10),
    memory: 62.4,
    gpu: 38,
    diskRead: generateTimeData(20, 120, 60, 0, 500),
    diskWrite: generateTimeData(20, 80, 40, 0, 500),
    netUp: generateTimeData(40, 15, 8, 0, 100),
    netDown: generateTimeData(40, 45, 20, 0, 200),
    temp: 52,
    power: 18.5,
  }));

  // 实时更新
  React.useEffect(() => {
    const timer = setInterval(() => {
      setData(prev => {
        const pushShift = (arr, val) => { const n = [...arr.slice(1), val]; return n; };
        const rand = (base, varr, min = 0, max = 100) => {
          let v = base + (Math.random() - 0.5) * varr * 2;
          return Math.max(min, Math.min(max, +v.toFixed(1)));
        };
        return {
          pCore: pushShift(prev.pCore, rand(45, 15)),
          eCore: pushShift(prev.eCore, rand(25, 10)),
          memory: rand(prev.memory, 2, 40, 85),
          gpu: rand(prev.gpu, 8, 10, 90),
          diskRead: pushShift(prev.diskRead, rand(120, 80, 10, 800)),
          diskWrite: pushShift(prev.diskWrite, rand(80, 50, 5, 600)),
          netUp: pushShift(prev.netUp, rand(15, 10, 0, 100)),
          netDown: pushShift(prev.netDown, rand(45, 25, 0, 200)),
          temp: rand(prev.temp, 1.5, 40, 75),
          power: rand(prev.power, 2, 8, 35),
        };
      });
    }, 1800);
    return () => clearInterval(timer);
  }, []);

  const xLabels = React.useMemo(() => generateXAxisTime(40), []);
  const xLabelsShort = React.useMemo(() => generateXAxisTime(20), []);

  // CPU 折线图 - P-Core vs E-Core
  const cpuChartOption = React.useMemo(() => ({
    grid: { top: 28, left: 36, right: 12, bottom: 22 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: 'rgba(20,22,36,0.95)',
      borderColor: 'rgba(255,255,255,0.1)',
      textStyle: { color: '#e6e8ef', fontSize: 11 },
      axisPointer: { type: 'line', lineStyle: { color: 'rgba(255,255,255,0.1)' } },
    },
    legend: {
      data: ['性能核心', '效率核心'],
      top: 0, right: 0,
      textStyle: { color: 'rgba(230,232,239,0.7)', fontSize: 11 },
      itemWidth: 10, itemHeight: 10,
    },
    xAxis: {
      type: 'category', data: xLabels,
      axisLine: { lineStyle: { color: 'rgba(255,255,255,0.08)' } },
      axisTick: { show: false },
      axisLabel: { color: 'rgba(230,232,239,0.4)', fontSize: 10, showMaxLabel: false, interval: 7 },
    },
    yAxis: {
      type: 'value', min: 0, max: 100,
      splitLine: { lineStyle: { color: 'rgba(255,255,255,0.05)' } },
      axisLabel: { color: 'rgba(230,232,239,0.4)', fontSize: 10, formatter: '{value}%' },
    },
    series: [
      {
        name: '性能核心', type: 'line', smooth: true, showSymbol: false,
        data: data.pCore,
        lineStyle: { width: 2, color: '#0A84FF' },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(10,132,255,0.35)' },
            { offset: 1, color: 'rgba(10,132,255,0.02)' },
          ]),
        },
      },
      {
        name: '效率核心', type: 'line', smooth: true, showSymbol: false,
        data: data.eCore,
        lineStyle: { width: 2, color: '#30D158' },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(48,209,88,0.25)' },
            { offset: 1, color: 'rgba(48,209,88,0.02)' },
          ]),
        },
      },
    ],
  }), [data.pCore, data.eCore, xLabels]);

  // 网络上下行
  const netChartOption = React.useMemo(() => ({
    grid: { top: 28, left: 36, right: 12, bottom: 22 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: 'rgba(20,22,36,0.95)',
      borderColor: 'rgba(255,255,255,0.1)',
      textStyle: { color: '#e6e8ef', fontSize: 11 },
    },
    legend: {
      data: ['下载', '上传'],
      top: 0, right: 0,
      textStyle: { color: 'rgba(230,232,239,0.7)', fontSize: 11 },
      itemWidth: 10, itemHeight: 10,
    },
    xAxis: {
      type: 'category', data: xLabels,
      axisLine: { lineStyle: { color: 'rgba(255,255,255,0.08)' } },
      axisTick: { show: false },
      axisLabel: { color: 'rgba(230,232,239,0.4)', fontSize: 10, interval: 7 },
    },
    yAxis: {
      type: 'value',
      splitLine: { lineStyle: { color: 'rgba(255,255,255,0.05)' } },
      axisLabel: { color: 'rgba(230,232,239,0.4)', fontSize: 10, formatter: '{value} MB/s' },
    },
    series: [
      {
        name: '下载', type: 'line', smooth: true, showSymbol: false,
        data: data.netDown,
        lineStyle: { width: 2, color: '#BF5AF2' },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(191,90,242,0.3)' },
            { offset: 1, color: 'rgba(191,90,242,0.02)' },
          ]),
        },
      },
      {
        name: '上传', type: 'line', smooth: true, showSymbol: false,
        data: data.netUp,
        lineStyle: { width: 2, color: '#FF9F0A' },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(255,159,10,0.25)' },
            { offset: 1, color: 'rgba(255,159,10,0.02)' },
          ]),
        },
      },
    ],
  }), [data.netUp, data.netDown, xLabels]);

  // 磁盘 I/O 柱状图
  const diskChartOption = React.useMemo(() => ({
    grid: { top: 28, left: 36, right: 12, bottom: 22 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: 'rgba(20,22,36,0.95)',
      borderColor: 'rgba(255,255,255,0.1)',
      textStyle: { color: '#e6e8ef', fontSize: 11 },
    },
    legend: {
      data: ['读取', '写入'],
      top: 0, right: 0,
      textStyle: { color: 'rgba(230,232,239,0.7)', fontSize: 11 },
      itemWidth: 10, itemHeight: 10,
    },
    xAxis: {
      type: 'category', data: xLabelsShort,
      axisLine: { lineStyle: { color: 'rgba(255,255,255,0.08)' } },
      axisTick: { show: false },
      axisLabel: { color: 'rgba(230,232,239,0.4)', fontSize: 10, interval: 4 },
    },
    yAxis: {
      type: 'value',
      splitLine: { lineStyle: { color: 'rgba(255,255,255,0.05)' } },
      axisLabel: { color: 'rgba(230,232,239,0.4)', fontSize: 10, formatter: '{value} MB/s' },
    },
    series: [
      { name: '读取', type: 'bar', data: data.diskRead, itemStyle: { color: '#64D2FF', borderRadius: [2, 2, 0, 0] }, barWidth: 6 },
      { name: '写入', type: 'bar', data: data.diskWrite, itemStyle: { color: '#FFD60A', borderRadius: [2, 2, 0, 0] }, barWidth: 6 },
    ],
  }), [data.diskRead, data.diskWrite, xLabelsShort]);

  // 内存环形图
  const memoryOption = React.useMemo(() => ({
    series: [{
      type: 'gauge',
      startAngle: 210, endAngle: -30,
      min: 0, max: 16,
      radius: '90%',
      center: ['50%', '55%'],
      progress: { show: true, width: 14, roundCap: true, itemStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 1, 0, [
          { offset: 0, color: '#0A84FF' },
          { offset: 1, color: '#BF5AF2' },
        ]),
      }},
      axisLine: { lineStyle: { width: 14, color: [[1, 'rgba(255,255,255,0.08)']] } },
      splitLine: { show: false },
      axisTick: { show: false },
      axisLabel: { show: false },
      pointer: { show: false },
      anchor: { show: false },
      detail: {
        valueAnimation: true,
        offsetCenter: [0, '-5%'],
        fontSize: 28, fontWeight: 600,
        color: '#e6e8ef',
        formatter: (v) => (v * 16 / 100 * 0 + data.memory * 0.16).toFixed(1) + ' GB',
      },
      title: {
        offsetCenter: [0, '45%'],
        fontSize: 11, color: 'rgba(230,232,239,0.5)',
      },
      data: [{ value: data.memory * 0.16, name: '统一内存 Unified Memory' }],
    }],
  }), [data.memory]);

  const processes = [
    { name: 'Code Helper (Renderer)', cpu: 12.4, mem: 842, pid: 3421 },
    { name: 'Google Chrome', cpu: 8.7, mem: 1240, pid: 1245 },
    { name: 'node', cpu: 6.2, mem: 312, pid: 5678 },
    { name: 'Docker', cpu: 4.1, mem: 1820, pid: 2341 },
    { name: 'Terminal', cpu: 2.3, mem: 48, pid: 892 },
    { name: 'Finder', cpu: 1.5, mem: 86, pid: 312 },
    { name: 'Activity Monitor', cpu: 1.1, mem: 52, pid: 2345 },
    { name: 'Spotlight', cpu: 0.8, mem: 76, pid: 235 },
  ];

  if (compact) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: 12, height: '100%', padding: 4 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '0 4px' }}>
          <Icon name="cpu" size={16} color="#0A84FF" />
          <span style={{ fontSize: 13, fontWeight: 600 }}>系统监控 · Apple M1</span>
        </div>
        <div className="glass-light" style={{ borderRadius: 10, padding: 10, height: 140 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
            <span style={{ fontSize: 11, color: 'rgba(230,232,239,0.6)' }}>CPU 使用率</span>
            <span className="mono" style={{ fontSize: 11, color: '#0A84FF' }}>{data.pCore[data.pCore.length-1] + data.eCore[data.eCore.length-1] / 2}%</span>
          </div>
          <EChart option={cpuChartOption} style={{ height: 100 }} />
        </div>
        <div className="glass-light" style={{ borderRadius: 10, padding: '10px 12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 6 }}>
            <span style={{ fontSize: 11, color: 'rgba(230,232,239,0.6)' }}>统一内存</span>
            <span className="mono" style={{ fontSize: 11, color: '#BF5AF2' }}>{(data.memory * 0.16).toFixed(1)} GB / 16 GB</span>
          </div>
          <div style={{ height: 6, background: 'rgba(255,255,255,0.08)', borderRadius: 3, overflow: 'hidden' }}>
            <div style={{
              width: `${data.memory}%`, height: '100%',
              background: 'linear-gradient(90deg, #0A84FF, #BF5AF2)',
              borderRadius: 3,
            }} />
          </div>
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
          <div className="glass-light" style={{ borderRadius: 10, padding: 10 }}>
            <div style={{ fontSize: 10, color: 'rgba(230,232,239,0.5)', marginBottom: 4 }}>GPU (Metal)</div>
            <div className="mono" style={{ fontSize: 16, fontWeight: 600, color: '#30D158' }}>{data.gpu.toFixed(1)}%</div>
          </div>
          <div className="glass-light" style={{ borderRadius: 10, padding: 10 }}>
            <div style={{ fontSize: 10, color: 'rgba(230,232,239,0.5)', marginBottom: 4 }}>温度</div>
            <div className="mono" style={{ fontSize: 16, fontWeight: 600, color: '#FF9F0A' }}>{data.temp.toFixed(0)}°C</div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div style={{ padding: 20, overflow: 'auto', height: '100%', display: 'flex', flexDirection: 'column', gap: 16 }}>
      {/* 头部 */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <div style={{
            width: 40, height: 40, borderRadius: 10,
            background: 'linear-gradient(135deg, #0A84FF 0%, #BF5AF2 100%)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
          }}>
            <Icon name="cpu" size={22} color="#fff" />
          </div>
          <div>
            <h2 style={{ fontSize: 18, fontWeight: 700 }}>系统监控</h2>
            <p style={{ fontSize: 12, color: 'rgba(230,232,239,0.5)' }}>Apple M1 · 8核 CPU · 8核 GPU · 16GB 统一内存</p>
          </div>
        </div>
        <div style={{ display: 'flex', gap: 8 }}>
          <div className="glass-light" style={{ padding: '6px 12px', borderRadius: 8, display: 'flex', alignItems: 'center', gap: 6 }}>
            <div style={{ width: 6, height: 6, borderRadius: '50%', background: '#30D158' }} />
            <span style={{ fontSize: 11, color: 'rgba(230,232,239,0.7)' }}>低功耗模式</span>
          </div>
        </div>
      </div>

      {/* 主指标网格 */}
      <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr 1fr', gap: 12 }}>
        {/* CPU 大图 */}
        <div className="glass-light" style={{ borderRadius: 14, padding: 16, height: 220 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <Icon name="cpu" size={16} color="#0A84FF" />
              <span style={{ fontSize: 13, fontWeight: 600 }}>CPU 使用率</span>
            </div>
            <div style={{ display: 'flex', gap: 12 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                <span style={{ width: 8, height: 8, borderRadius: 2, background: '#0A84FF' }} />
                <span className="mono" style={{ fontSize: 11, color: 'rgba(230,232,239,0.6)' }}>P-Core {data.pCore[data.pCore.length-1].toFixed(1)}%</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                <span style={{ width: 8, height: 8, borderRadius: 2, background: '#30D158' }} />
                <span className="mono" style={{ fontSize: 11, color: 'rgba(230,232,239,0.6)' }}>E-Core {data.eCore[data.eCore.length-1].toFixed(1)}%</span>
              </div>
            </div>
          </div>
          <EChart option={cpuChartOption} style={{ height: 160 }} />
        </div>

        {/* 内存环形图 */}
        <div className="glass-light" style={{ borderRadius: 14, padding: 16, height: 220, display: 'flex', flexDirection: 'column' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
            <Icon name="memory" size={16} color="#BF5AF2" />
            <span style={{ fontSize: 13, fontWeight: 600 }}>内存压力</span>
          </div>
          <div style={{ flex: 1, position: 'relative' }}>
            <EChart option={memoryOption} style={{ height: '100%' }} />
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 10, color: 'rgba(230,232,239,0.5)', marginTop: -10 }}>
            <span>应用 {7.2} GB</span>
            <span>缓存 {2.7} GB</span>
          </div>
        </div>

        {/* GPU + 温度 + 功耗 */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <div className="glass-light" style={{ borderRadius: 14, padding: 14, flex: 1 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 10 }}>
              <Icon name="gpu" size={16} color="#30D158" />
              <span style={{ fontSize: 12, fontWeight: 600 }}>GPU · Metal 加速</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', marginBottom: 8 }}>
              <span className="mono" style={{ fontSize: 24, fontWeight: 700, color: '#30D158' }}>{data.gpu.toFixed(1)}</span>
              <span style={{ fontSize: 12, color: 'rgba(230,232,239,0.5)' }}>%</span>
            </div>
            <div style={{ height: 6, background: 'rgba(255,255,255,0.08)', borderRadius: 3, overflow: 'hidden' }}>
              <div style={{ width: `${data.gpu}%`, height: '100%', background: '#30D158', borderRadius: 3, transition: 'width 1.8s ease' }} />
            </div>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10 }}>
            <div className="glass-light" style={{ borderRadius: 12, padding: 12 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 6 }}>
                <Icon name="thermometer" size={14} color="#FF9F0A" />
                <span style={{ fontSize: 10, color: 'rgba(230,232,239,0.5)' }}>温度</span>
              </div>
              <div className="mono" style={{ fontSize: 18, fontWeight: 700, color: '#FF9F0A' }}>{data.temp.toFixed(0)}°</div>
            </div>
            <div className="glass-light" style={{ borderRadius: 12, padding: 12 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 6 }}>
                <Icon name="zap" size={14} color="#FFD60A" />
                <span style={{ fontSize: 10, color: 'rgba(230,232,239,0.5)' }}>功耗</span>
              </div>
              <div className="mono" style={{ fontSize: 18, fontWeight: 700, color: '#FFD60A' }}>{data.power.toFixed(1)}W</div>
            </div>
          </div>
        </div>
      </div>

      {/* 第二行：磁盘 + 网络 */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
        <div className="glass-light" style={{ borderRadius: 14, padding: 16, height: 200 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <Icon name="hardDrive" size={16} color="#64D2FF" />
              <span style={{ fontSize: 13, fontWeight: 600 }}>磁盘 I/O</span>
            </div>
            <div style={{ display: 'flex', gap: 12 }}>
              <span className="mono" style={{ fontSize: 11, color: '#64D2FF' }}>读 {data.diskRead[data.diskRead.length-1].toFixed(0)} MB/s</span>
              <span className="mono" style={{ fontSize: 11, color: '#FFD60A' }}>写 {data.diskWrite[data.diskWrite.length-1].toFixed(0)} MB/s</span>
            </div>
          </div>
          <EChart option={diskChartOption} style={{ height: 140 }} />
        </div>

        <div className="glass-light" style={{ borderRadius: 14, padding: 16, height: 200 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <Icon name="wifi" size={16} color="#BF5AF2" />
              <span style={{ fontSize: 13, fontWeight: 600 }}>网络流量</span>
            </div>
            <div style={{ display: 'flex', gap: 12 }}>
              <span className="mono" style={{ fontSize: 11, color: '#BF5AF2' }}>↓ {data.netDown[data.netDown.length-1].toFixed(1)} MB/s</span>
              <span className="mono" style={{ fontSize: 11, color: '#FF9F0A' }}>↑ {data.netUp[data.netUp.length-1].toFixed(1)} MB/s</span>
            </div>
          </div>
          <EChart option={netChartOption} style={{ height: 140 }} />
        </div>
      </div>

      {/* 进程列表 */}
      <div className="glass-light" style={{ borderRadius: 14, padding: 16, flex: 1, minHeight: 200 }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <Icon name="activity" size={16} color="#FF6B6B" />
            <span style={{ fontSize: 13, fontWeight: 600 }}>进程列表</span>
            <span style={{ fontSize: 11, color: 'rgba(230,232,239,0.4)', marginLeft: 4 }}>247 个进程</span>
          </div>
          <div style={{ display: 'flex', gap: 6 }}>
            <button style={filterBtnStyle(true)}>CPU</button>
            <button style={filterBtnStyle(false)}>内存</button>
            <button style={filterBtnStyle(false)}>名称</button>
          </div>
        </div>
        <div style={{ overflow: 'auto' }}>
          <table style={{ width: '100%', fontSize: 12, borderCollapse: 'collapse' }}>
            <thead>
              <tr style={{ color: 'rgba(230,232,239,0.4)', textAlign: 'left' }}>
                <th style={{ padding: '8px 12px', fontWeight: 500, fontSize: 11 }}>进程名称</th>
                <th style={{ padding: '8px 12px', fontWeight: 500, fontSize: 11, textAlign: 'right' }}>PID</th>
                <th style={{ padding: '8px 12px', fontWeight: 500, fontSize: 11, textAlign: 'right' }}>CPU %</th>
                <th style={{ padding: '8px 12px', fontWeight: 500, fontSize: 11, textAlign: 'right' }}>内存 (MB)</th>
              </tr>
            </thead>
            <tbody>
              {processes.map((p, i) => (
                <tr key={i} style={{ borderTop: '1px solid rgba(255,255,255,0.04)' }}>
                  <td style={{ padding: '8px 12px', display: 'flex', alignItems: 'center', gap: 8 }}>
                    <div style={{ width: 18, height: 18, borderRadius: 4, background: 'rgba(255,255,255,0.08)' }} />
                    <span>{p.name}</span>
                  </td>
                  <td style={{ padding: '8px 12px', textAlign: 'right', color: 'rgba(230,232,239,0.5)', fontFamily: 'JetBrains Mono, monospace' }}>{p.pid}</td>
                  <td style={{ padding: '8px 12px', textAlign: 'right' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, justifyContent: 'flex-end' }}>
                      <div style={{ width: 60, height: 4, background: 'rgba(255,255,255,0.08)', borderRadius: 2, overflow: 'hidden' }}>
                        <div style={{ width: `${p.cpu * 3}%`, height: '100%', background: p.cpu > 10 ? '#FF6B6B' : p.cpu > 5 ? '#FF9F0A' : '#30D158', borderRadius: 2 }} />
                      </div>
                      <span className="mono" style={{ width: 45, textAlign: 'right' }}>{p.cpu.toFixed(1)}%</span>
                    </div>
                  </td>
                  <td style={{ padding: '8px 12px', textAlign: 'right', fontFamily: 'JetBrains Mono, monospace' }}>{p.mem}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

const filterBtnStyle = (active) => ({
  padding: '4px 10px',
  fontSize: 11,
  borderRadius: 6,
  border: 'none',
  cursor: 'pointer',
  background: active ? 'rgba(10,132,255,0.2)' : 'rgba(255,255,255,0.06)',
  color: active ? '#0A84FF' : 'rgba(230,232,239,0.6)',
  fontWeight: active ? 600 : 400,
});

Object.assign(window, { SystemMonitor, EChart, generateTimeData, generateXAxisTime });
