// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! 策略工作台主产品 shell 与共享只读状态渲染资源。

pub(super) const STRATEGY_WORKBENCH_HTML: &str = r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="icon" href="data:,">
  <title>NTPRO 策略工作台</title>
  <link rel="stylesheet" href="/assets/strategy-workbench.css">
</head>
<body>
  <div id="strategy-workbench" class="app-shell drawer-open">
    <aside class="left-rail">
      <div class="brand"><span class="brand-mark">NT</span><div><strong>NTPRO</strong><small>策略工作台</small></div></div>
      <nav id="primary-nav" aria-label="策略工作台主导航">
        <button class="nav-item active" type="button" data-section="总览"><span>总</span>总览</button>
        <button class="nav-item" type="button" data-section="策略"><span>策</span>策略</button>
        <button class="nav-item" type="button" data-section="Backtest"><span>测</span>Backtest</button>
        <button class="nav-item" type="button" data-section="Demo"><span>演</span>Demo</button>
        <button class="nav-item disabled" type="button" disabled title="Live 产品能力尚未开放"><span>实</span>Live<em>未开放</em></button>
        <button class="nav-item" type="button" data-section="运行"><span>运</span>运行</button>
        <button class="nav-item" type="button" data-section="数据"><span>数</span>数据</button>
        <button class="nav-item" type="button" data-section="风险"><span>险</span>风险</button>
        <button class="nav-item" type="button" data-section="系统状态"><span>态</span>系统状态</button>
      </nav>
      <div class="rail-status"><span id="rail-dot" class="status-dot"></span><div><strong id="rail-state">读取中</strong><small>Live 权限关闭</small></div></div>
    </aside>

    <section class="stage">
      <header class="topbar">
        <div class="scope"><span>策略</span><strong id="strategy-name">策略未加载</strong></div>
        <div class="scope"><span>版本</span><strong id="strategy-version">未知</strong></div>
        <div class="scope"><span>模式</span><strong id="environment">Demo</strong></div>
        <div class="scope optional"><span>账户</span><strong id="account-id">未知</strong></div>
        <div class="scope optional"><span>Venue</span><strong id="venue-id">未知</strong></div>
        <div class="top-actions">
          <span id="top-health" class="health-chip">正在连接</span>
          <button id="refresh" class="icon-button" type="button" title="刷新共享状态" aria-label="刷新共享状态">↻</button>
        </div>
      </header>

      <nav class="mode-tabs" aria-label="策略运行模式">
        <button type="button" data-mode="Backtest"><span class="mode-dot complete"></span>Backtest<small>历史验证</small></button>
        <button class="active" type="button" data-mode="Demo"><span class="mode-dot running"></span>Demo<small id="demo-mode-state">读取中</small></button>
        <button class="disabled" type="button" disabled title="真实 Live 产品能力尚未开放"><span class="mode-dot blocked"></span>Live<small>未开放</small></button>
        <button type="button" data-mode="compare"><span class="mode-dot neutral"></span>运行对比<small>框架预留</small></button>
      </nav>

      <div class="workarea">
        <main class="canvas">
          <section class="canvas-heading">
            <div><span id="section-label" class="eyebrow">策略总览</span><h1 id="run-title">等待策略运行状态</h1><p id="run-subtitle">同一策略版本贯穿 Backtest、Demo 和 Live；当前页面只读取已冻结的 Demo 基线。</p></div>
            <span id="readiness-badge" class="badge blocked">交易准备度阻断</span>
          </section>

          <section id="connection-banner" class="connection-banner loading" aria-live="polite">
            <div><strong id="connection-title">正在读取共享状态</strong><span id="connection-detail">旧状态已清空，等待只读合同验证</span></div>
            <span id="connection-state">读取中</span>
          </section>

          <section class="metric-grid" aria-label="当前运行摘要">
            <article><span>当前 Run</span><strong id="metric-run">未加载</strong><small id="metric-runtime">状态未知</small></article>
            <article><span>研究引用</span><strong id="metric-research">未验证</strong><small id="metric-backtest">Backtest 引用未知</small></article>
            <article><span>技术健康</span><strong id="metric-health">未知</strong><small id="metric-freshness">时效未知</small></article>
            <article><span>Live 准入</span><strong>未开放</strong><small class="warning">真实交易权限为 false</small></article>
          </section>

          <div class="main-grid">
            <section class="panel run-panel">
              <header><div><span class="eyebrow">三模式闭环</span><h2>当前版本运行状态</h2></div><span id="run-contract">只读桥接</span></header>
              <div class="mode-progress">
                <article class="complete"><span>01</span><div><strong>Backtest</strong><small id="backtest-summary">等待历史引用</small></div><em>历史</em></article>
                <article class="active"><span>02</span><div><strong>Demo</strong><small id="demo-summary">等待 Sandbox 状态</small></div><em>当前</em></article>
                <article class="blocked"><span>03</span><div><strong>Live</strong><small>真实适配器、账户与权限未开放</small></div><em>阻断</em></article>
              </div>
              <div class="run-table-wrap">
                <table>
                  <thead><tr><th>运行</th><th>模式</th><th>状态</th><th>账户</th><th>Venue</th><th>来源</th></tr></thead>
                  <tbody id="run-table-body"><tr><td colspan="6" class="empty">等待共享状态</td></tr></tbody>
                </table>
              </div>
            </section>

            <section class="panel readiness-panel">
              <header><div><span class="eyebrow">当前判断</span><h2>状态与边界</h2></div></header>
              <div id="axis-list" class="axis-list"><p class="empty">等待共享状态</p></div>
            </section>
          </div>
        </main>

        <aside class="inspector" aria-label="当前运行详情">
          <button id="drawer-toggle" class="drawer-toggle" type="button" title="收起详情栏" aria-label="收起详情栏" aria-expanded="true">›</button>
          <div class="inspector-body">
            <header><span class="eyebrow">当前选择</span><strong id="inspector-title">Run 未加载</strong></header>
            <div id="inspector-kv" class="kv-list"><p class="empty">等待共享状态</p></div>
            <section><span class="eyebrow">准入边界</span><div id="boundary-list" class="boundary-list"><p class="empty">等待验证</p></div></section>
            <section><span class="eyebrow">数据来源</span><div id="source-list" class="source-list"><p class="empty">等待验证</p></div></section>
          </div>
        </aside>
      </div>

      <section class="bottom-dock" aria-label="策略运行活动区">
        <div class="dock-tabs" role="tablist">
          <button class="active" type="button" role="tab" aria-selected="true" data-dock="positions">持仓</button>
          <button type="button" role="tab" aria-selected="false" data-dock="activity">活动</button>
          <button type="button" role="tab" aria-selected="false" data-dock="fills">成交</button>
          <button type="button" role="tab" aria-selected="false" data-dock="logs">日志</button>
        </div>
        <div id="dock-content" class="dock-content"><p class="empty">等待共享状态</p></div>
      </section>

      <footer class="statusbar">
        <span id="status-data" class="warning">数据：未知</span>
        <span id="status-account">账户：未知</span>
        <span id="status-venue">Venue：未知</span>
        <span id="status-risk" class="warning">风险：阻断</span>
        <span id="status-node">节点：未知</span>
        <span id="status-updated">更新：未知</span>
      </footer>
    </section>
  </div>
  <script src="/assets/strategy-workbench.js"></script>
</body>
</html>
"#;

pub(super) const STRATEGY_WORKBENCH_CSS: &str = r#":root {
  color-scheme: dark;
  font-family: Inter, "Noto Sans CJK SC", "Noto Sans CJK", ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  color: #e7eee9;
  background: #0d1411;
}

* { box-sizing: border-box; }
html, body { min-width: 320px; min-height: 100%; }
body { margin: 0; background: #0d1411; }
button { font: inherit; letter-spacing: 0; }
.app-shell { --bg: #0d1411; --raised: #14201b; --panel: #17201c; --line: #34423b; --muted: #91a39a; --text: #e7eee9; --green: #55e6ad; --blue: #6eb5ff; --amber: #e6a64c; --red: #ff8a82; display: grid; grid-template-columns: 176px minmax(0, 1fr); min-height: 100vh; background: var(--bg); color: var(--text); }
.left-rail { position: sticky; top: 0; display: flex; height: 100vh; min-width: 0; flex-direction: column; border-right: 1px solid var(--line); background: var(--raised); }
.brand { display: flex; min-height: 72px; align-items: center; gap: 10px; padding: 0 16px; border-bottom: 1px solid var(--line); }
.brand-mark { display: grid; width: 32px; height: 32px; place-items: center; border: 1px solid #466257; border-radius: 4px; color: var(--green); font-size: 11px; font-weight: 900; }
.brand strong, .brand small { display: block; }
.brand strong { font-size: 13px; }
.brand small { margin-top: 3px; color: var(--muted); font-size: 10px; }
#primary-nav { display: grid; align-content: start; gap: 2px; flex: 1; padding: 12px 8px; }
.nav-item { position: relative; display: grid; min-height: 40px; grid-template-columns: 24px minmax(0, 1fr); align-items: center; gap: 9px; padding: 6px 9px; border: 0; border-left: 2px solid transparent; color: var(--muted); background: transparent; text-align: left; cursor: pointer; }
.nav-item > span { display: grid; width: 24px; height: 22px; place-items: center; border: 1px solid #46574f; border-radius: 3px; color: #b7c4bd; font-size: 10px; font-weight: 800; }
.nav-item:hover, .nav-item.active { border-left-color: var(--green); color: #fff; background: #26342e; }
.nav-item.active > span { border-color: var(--green); color: #fff; background: #1f3028; }
.nav-item.disabled { cursor: not-allowed; opacity: .58; }
.nav-item em { position: absolute; right: 8px; color: var(--red); font-size: 8px; font-style: normal; }
.rail-status { display: flex; align-items: center; gap: 8px; min-height: 58px; margin: 0 10px 10px; padding: 10px 8px; border-top: 1px solid var(--line); }
.rail-status div { display: grid; gap: 2px; }
.rail-status strong { font-size: 10px; }
.rail-status small { color: var(--muted); font-size: 8px; }
.status-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--amber); }
.status-dot.ready { background: var(--green); box-shadow: 0 0 7px rgba(85, 230, 173, .5); }
.status-dot.blocked { background: var(--red); }
.stage { display: grid; min-width: 0; grid-template-rows: 54px 42px minmax(380px, 1fr) 184px 26px; height: 100vh; overflow: hidden; }
.topbar { display: flex; min-width: 0; align-items: stretch; border-bottom: 1px solid var(--line); background: #111a16; }
.scope { display: flex; min-width: 112px; max-width: 190px; justify-content: center; flex-direction: column; padding: 0 13px; border-right: 1px solid var(--line); }
.scope span { color: var(--muted); font-size: 9px; }
.scope strong { margin-top: 4px; overflow: hidden; color: #fff; font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
.top-actions { display: flex; align-items: center; gap: 8px; margin-left: auto; padding: 0 12px; }
.health-chip { padding: 5px 8px; border: 1px solid #665336; border-radius: 3px; color: var(--amber); font-size: 9px; white-space: nowrap; }
.health-chip.ready { border-color: #286c53; color: var(--green); }
.health-chip.blocked { border-color: #75423e; color: var(--red); }
.icon-button { display: grid; width: 30px; height: 30px; place-items: center; padding: 0; border: 1px solid var(--line); border-radius: 3px; color: #dce6e0; background: var(--panel); cursor: pointer; }
.icon-button:disabled { cursor: wait; opacity: .55; }
.mode-tabs { display: flex; min-width: 0; overflow-x: auto; border-bottom: 1px solid var(--line); background: #111a16; }
.mode-tabs button { display: grid; min-width: 150px; grid-template-columns: 8px auto; grid-template-rows: 18px 14px; align-items: center; column-gap: 7px; padding: 4px 14px; border: 0; border-right: 1px solid var(--line); color: var(--muted); background: transparent; text-align: left; cursor: pointer; }
.mode-tabs button.active { color: #fff; box-shadow: inset 0 2px 0 var(--green); background: #202d27; }
.mode-tabs button.disabled { cursor: not-allowed; opacity: .58; }
.mode-tabs button small { grid-column: 2; color: #71847a; font-size: 8px; }
.mode-dot { width: 6px; height: 6px; border-radius: 50%; background: #71847a; }
.mode-dot.complete { background: var(--blue); }.mode-dot.running { background: var(--green); }.mode-dot.blocked { background: var(--red); }
.workarea { display: grid; min-width: 0; grid-template-columns: minmax(0, 1fr) 0; overflow: hidden; transition: grid-template-columns 160ms ease; }
.app-shell.drawer-open .workarea { grid-template-columns: minmax(0, 1fr) 248px; }
.canvas { min-width: 0; overflow: auto; padding: 14px; }
.canvas-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 10px; }
.eyebrow { color: var(--green); font-size: 9px; font-weight: 800; }
.canvas-heading h1 { margin: 4px 0 0; font-size: 18px; letter-spacing: 0; }
.canvas-heading p { max-width: 760px; margin: 5px 0 0; color: var(--muted); font-size: 10px; }
.badge { flex: 0 0 auto; padding: 6px 8px; border: 1px solid #75423e; border-radius: 3px; color: var(--red); font-size: 9px; font-weight: 800; }
.connection-banner { display: flex; min-height: 44px; align-items: center; justify-content: space-between; gap: 14px; margin-bottom: 10px; padding: 7px 10px; border: 1px solid #665336; border-left: 3px solid var(--amber); background: #171d19; }
.connection-banner > div { display: grid; gap: 2px; }
.connection-banner strong { font-size: 10px; }.connection-banner span { color: var(--muted); font-size: 8px; }
.connection-banner > span { color: var(--amber); font-weight: 800; }
.connection-banner.ready { border-color: #286c53; border-left-color: var(--green); }.connection-banner.ready > span { color: var(--green); }
.connection-banner.blocked { border-color: #75423e; border-left-color: var(--red); }.connection-banner.blocked > span { color: var(--red); }
.metric-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); border: 1px solid var(--line); background: var(--panel); }
.metric-grid article { min-width: 0; min-height: 68px; padding: 10px; border-right: 1px solid var(--line); }.metric-grid article:last-child { border-right: 0; }
.metric-grid span { color: var(--muted); font-size: 9px; }.metric-grid strong { display: block; margin-top: 7px; overflow-wrap: anywhere; color: #fff; font-size: 14px; }.metric-grid small { display: block; margin-top: 4px; color: var(--green); font-size: 8px; overflow-wrap: anywhere; }.metric-grid .warning { color: var(--amber); }
.main-grid { display: grid; grid-template-columns: minmax(0, 1.55fr) minmax(220px, .45fr); gap: 10px; margin-top: 10px; }
.panel { min-width: 0; border: 1px solid var(--line); background: var(--panel); }
.panel > header { display: flex; min-height: 42px; align-items: center; justify-content: space-between; gap: 10px; padding: 7px 10px; border-bottom: 1px solid var(--line); }
.panel h2 { margin: 3px 0 0; font-size: 11px; letter-spacing: 0; }.panel header > span { color: var(--muted); font-size: 8px; }
.mode-progress { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); border-bottom: 1px solid var(--line); }
.mode-progress article { display: grid; min-width: 0; grid-template-columns: 26px minmax(0, 1fr) auto; align-items: center; gap: 7px; min-height: 66px; padding: 8px; border-right: 1px solid var(--line); }.mode-progress article:last-child { border-right: 0; }
.mode-progress article > span { display: grid; width: 24px; height: 24px; place-items: center; border: 1px solid #456354; border-radius: 50%; color: var(--green); font-size: 8px; }
.mode-progress strong, .mode-progress small { display: block; overflow-wrap: anywhere; }.mode-progress strong { font-size: 9px; }.mode-progress small { margin-top: 4px; color: var(--muted); font-size: 8px; }.mode-progress em { color: var(--muted); font-size: 8px; font-style: normal; }.mode-progress .blocked > span, .mode-progress .blocked em { color: var(--red); border-color: #75423e; }
.run-table-wrap { max-width: 100%; overflow-x: auto; }
table { width: 100%; border-collapse: collapse; table-layout: fixed; }
th, td { padding: 8px 9px; overflow: hidden; border-bottom: 1px solid #303e37; color: #a6b7ae; font-size: 8px; text-align: left; text-overflow: ellipsis; white-space: nowrap; }
th { color: var(--muted); background: #131c18; font-weight: 600; }.empty { color: #71847a; font-size: 9px; text-align: center; }
.axis-list { display: grid; }.axis-item { display: flex; min-height: 43px; align-items: center; justify-content: space-between; gap: 8px; padding: 7px 10px; border-bottom: 1px solid #303e37; }.axis-item:last-child { border-bottom: 0; }.axis-item div { display: grid; gap: 3px; }.axis-item strong { font-size: 9px; }.axis-item small { color: var(--muted); font-size: 8px; }.axis-item span { color: var(--green); font-size: 8px; }.axis-item span.blocked { color: var(--red); }
.inspector { position: relative; min-width: 0; border-left: 1px solid var(--line); background: var(--raised); }
.drawer-toggle { position: absolute; z-index: 2; top: 10px; left: -30px; display: grid; width: 29px; height: 29px; place-items: center; padding: 0; border: 1px solid var(--line); border-right: 0; border-radius: 3px 0 0 3px; color: #c8d4ce; background: var(--raised); cursor: pointer; }
.inspector-body { width: 248px; height: 100%; padding: 13px; overflow: auto; visibility: hidden; opacity: 0; transition: opacity 120ms ease; }.app-shell.drawer-open .inspector-body { visibility: visible; opacity: 1; }
.inspector-body header { display: grid; gap: 5px; padding-bottom: 11px; border-bottom: 1px solid var(--line); }.inspector-body header strong { overflow-wrap: anywhere; font-size: 11px; }.inspector-body section { margin-top: 16px; }
.kv-list { display: grid; }.kv-row { display: flex; justify-content: space-between; gap: 8px; padding: 8px 0; border-bottom: 1px solid #303e37; font-size: 8px; }.kv-row span { color: var(--muted); }.kv-row strong { color: #fff; overflow-wrap: anywhere; text-align: right; }
.boundary-list, .source-list { display: grid; margin-top: 6px; }.boundary-item, .source-item { padding: 7px 0; border-bottom: 1px solid #303e37; color: var(--muted); font-size: 8px; overflow-wrap: anywhere; }.boundary-item strong { display: block; margin-bottom: 3px; color: var(--green); }
.bottom-dock { min-width: 0; overflow: hidden; border-top: 1px solid var(--line); background: #111a16; }
.dock-tabs { display: flex; height: 34px; border-bottom: 1px solid var(--line); }.dock-tabs button { min-width: 100px; padding: 0 12px; border: 0; border-right: 1px solid var(--line); color: var(--muted); background: transparent; font-size: 9px; cursor: pointer; }.dock-tabs button.active { color: #fff; box-shadow: inset 0 -2px 0 var(--green); background: #202d27; }
.dock-content { height: 150px; overflow: auto; }.dock-content table { min-width: 680px; }.dock-content p { padding: 14px; margin: 0; }.dock-summary { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); }.dock-summary article { min-height: 70px; padding: 12px; border-right: 1px solid var(--line); }.dock-summary article:last-child { border-right: 0; }.dock-summary span, .dock-summary small { display: block; color: var(--muted); font-size: 8px; }.dock-summary strong { display: block; margin: 7px 0 4px; color: #fff; font-size: 11px; }
.statusbar { display: flex; min-width: 0; align-items: center; gap: 16px; padding: 0 10px; overflow-x: auto; border-top: 1px solid var(--line); color: var(--muted); background: var(--bg); font-size: 8px; white-space: nowrap; }.statusbar span::before { content: ""; display: inline-block; width: 5px; height: 5px; margin-right: 5px; border-radius: 50%; background: var(--green); }.statusbar span.warning::before { background: var(--amber); }.statusbar span.blocked::before { background: var(--red); }

@media (max-width: 1050px) {
  .app-shell { grid-template-columns: 64px minmax(0, 1fr); }.brand { justify-content: center; padding: 0; }.brand div, .nav-item:not(.disabled) { font-size: 0; }.nav-item { grid-template-columns: 24px; justify-content: center; padding: 6px; }.nav-item > span { font-size: 10px; }.nav-item em { display: none; }.rail-status div { display: none; }.rail-status { justify-content: center; }.scope.optional { display: none; }.app-shell.drawer-open .workarea { grid-template-columns: minmax(0, 1fr) 220px; }.inspector-body { width: 220px; }
}
@media (max-width: 760px) {
  .app-shell { display: block; }.left-rail { position: static; width: 100%; height: auto; border-right: 0; border-bottom: 1px solid var(--line); }.brand { display: none; }#primary-nav { display: flex; overflow-x: auto; padding: 6px; }.nav-item { min-width: 54px; min-height: 34px; border-left: 0; border-bottom: 2px solid transparent; }.nav-item:hover, .nav-item.active { border-left-color: transparent; border-bottom-color: var(--green); }.rail-status { display: none; }.stage { grid-template-rows: 48px 42px auto 180px 26px; height: auto; min-height: calc(100vh - 48px); overflow: visible; }.topbar { position: sticky; top: 0; z-index: 5; }.scope { min-width: 0; flex: 1; padding: 0 9px; }.scope:nth-of-type(n+4) { display: none; }.top-actions { padding: 0 7px; }.health-chip { display: none; }.mode-tabs button { min-width: 122px; }.workarea, .app-shell.drawer-open .workarea { grid-template-columns: minmax(0, 1fr); overflow: visible; }.canvas { overflow: visible; padding: 10px; }.inspector { position: fixed; z-index: 20; top: 0; right: 0; bottom: 0; width: min(86vw, 300px); transform: translateX(100%); border-left: 1px solid var(--line); transition: transform 160ms ease; }.app-shell.drawer-open .inspector { transform: translateX(0); }.inspector-body { width: 100%; visibility: visible; opacity: 1; }.drawer-toggle { left: -36px; width: 35px; }.metric-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }.metric-grid article:nth-child(2) { border-right: 0; }.metric-grid article:nth-child(-n+2) { border-bottom: 1px solid var(--line); }.main-grid { grid-template-columns: 1fr; }.mode-progress { grid-template-columns: 1fr; }.mode-progress article { border-right: 0; border-bottom: 1px solid var(--line); }.mode-progress article:last-child { border-bottom: 0; }.bottom-dock { min-height: 180px; }.statusbar { min-height: 26px; }
}
@media (max-width: 430px) {
  .scope:nth-of-type(3) { display: none; }.canvas-heading { display: grid; }.badge { justify-self: start; }.metric-grid { grid-template-columns: 1fr; }.metric-grid article { border-right: 0; border-bottom: 1px solid var(--line); }.metric-grid article:last-child { border-bottom: 0; }.dock-tabs button { min-width: 76px; padding: 0 8px; }.dock-summary { grid-template-columns: 1fr; }.dock-summary article { border-right: 0; border-bottom: 1px solid var(--line); }
}
"#;

pub(super) const STRATEGY_WORKBENCH_JS: &str = r#"const SHARED_STATUS_URL = "/api/mvp/v1/status";
const EXPECTED_SCHEMA = "ntpro.mvp_shared_status_api.response.v1";
const EXPECTED_CONTRACT = "ntpro.mvp_shared_status_api.v1";
const FALSE_BOUNDARIES = [
  "external_venue_connection", "order_submission_allowed", "order_mutation_allowed",
  "automatic_retry_allowed", "automatic_remediation_allowed", "real_orders_submitted",
];
const LABELS = {
  reference_bound: "已绑定", running: "运行中", stopped: "已停止", transitioning: "切换中",
  healthy: "健康", degraded: "降级", unhealthy: "不健康", not_running: "未运行",
  blocked: "阻断", fresh: "新鲜", stale: "陈旧", unknown: "未知", sandbox: "Demo / Sandbox",
};

const byId = (id) => document.getElementById(id);
const safe = (value) => value === null || value === undefined ? "未知" : String(value);
const escapeHtml = (value) => safe(value).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll("\"", "&quot;").replaceAll("'", "&#39;");
const label = (value) => LABELS[safe(value)] || safe(value);
const dashboardValue = (value) => value && typeof value === "object" ? value.value ?? value.availability ?? "未知" : "未知";

function requireObject(value, field) {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`共享状态缺少 ${field}`);
  return value;
}
function requireString(value, field) {
  if (typeof value !== "string" || value.trim().length === 0) throw new Error(`共享状态缺少 ${field}`);
  return value;
}
function requireBoundary(boundaries, field, expected, scope) {
  if (!Object.prototype.hasOwnProperty.call(boundaries, field) || boundaries[field] !== expected) throw new Error(`${scope} 边界异常：${field}`);
}
function requireAxis(axis, field, statuses) {
  requireObject(axis, field);
  if (!statuses.includes(axis.status)) throw new Error(`${field}.status 异常`);
  if (!["available", "missing", "error", "unknown"].includes(axis.availability)) throw new Error(`${field}.availability 异常`);
  if (!["fresh", "stale", "unknown"].includes(axis.freshness)) throw new Error(`${field}.freshness 异常`);
  if (!Array.isArray(axis.source_refs) || axis.source_refs.length === 0) throw new Error(`${field}.source_refs 缺失`);
  if (!Number.isSafeInteger(axis.observed_at_unix_ms) || axis.observed_at_unix_ms <= 0) throw new Error(`${field}.observed_at_unix_ms 异常`);
}
function validateSharedStatus(payload) {
  requireObject(payload, "payload");
  if (payload.schema_version !== EXPECTED_SCHEMA || payload.contract_version !== EXPECTED_CONTRACT) throw new Error("共享状态版本不匹配");
  if (!Array.isArray(payload.consumers) || !payload.consumers.includes("institution_workbench")) throw new Error("共享状态未授权当前策略用户视图");
  const identity = requireObject(payload.identity, "identity");
  const identities = requireObject(identity.identities, "identity.identities");
  const status = requireObject(payload.status, "status");
  const business = requireObject(payload.business, "business");
  const apiBoundaries = requireObject(payload.boundaries, "boundaries");
  const identityBoundaries = requireObject(identity.boundaries, "identity.boundaries");
  const statusBoundaries = requireObject(status.boundaries, "status.boundaries");
  for (const field of ["strategy_id", "strategy_version", "backtest_run_id", "backtest_result_ref", "node_id", "strategy_instance_id", "account_id", "venue_id", "environment"]) requireString(identities[field], `identity.identities.${field}`);
  if (identities.environment !== "sandbox") throw new Error("当前产品框架只允许 Demo / Sandbox 身份");
  if (identity.contract_id !== `${identities.node_id}:${identities.strategy_id}:${identities.strategy_instance_id}` || status.identity_contract_id !== identity.contract_id) throw new Error("策略运行身份不一致");
  requireBoundary(apiBoundaries, "read_only", true, "API");
  FALSE_BOUNDARIES.forEach((field) => {
    requireBoundary(apiBoundaries, field, false, "API");
    requireBoundary(identityBoundaries, field, false, "身份");
    requireBoundary(statusBoundaries, field, false, "状态");
  });
  requireBoundary(identityBoundaries, "read_only_product_contract", true, "身份");
  requireBoundary(statusBoundaries, "read_only_product_contract", true, "状态");
  requireAxis(status.research, "status.research", ["reference_bound"]);
  requireAxis(status.runtime, "status.runtime", ["running", "stopped", "transitioning", "unknown"]);
  requireAxis(status.technical_health, "status.technical_health", ["healthy", "degraded", "unhealthy", "not_running", "unknown"]);
  requireAxis(status.trading_readiness, "status.trading_readiness", ["blocked"]);
  if (!["available", "missing", "stale", "error", "identity_mismatch"].includes(business.availability)) throw new Error("business.availability 异常");
  if (!Array.isArray(payload.source_refs) || payload.source_refs.length === 0) throw new Error("source_refs 缺失");
  if (!Number.isSafeInteger(payload.generated_at_unix_ms) || payload.generated_at_unix_ms <= 0) throw new Error("generated_at_unix_ms 异常");
  return { identities, status, business, boundaries: apiBoundaries, sourceRefs: payload.source_refs, generatedAt: payload.generated_at_unix_ms };
}

let currentDock = "positions";
let currentProjection = null;
function resetSurface(message = "等待共享状态") {
  currentProjection = null;
  byId("strategy-name").textContent = "策略未加载";
  byId("strategy-version").textContent = "未知";
  byId("environment").textContent = "Demo";
  byId("account-id").textContent = "未知";
  byId("venue-id").textContent = "未知";
  byId("run-title").textContent = "等待策略运行状态";
  byId("metric-run").textContent = "未加载";
  byId("metric-runtime").textContent = "状态未知";
  byId("metric-research").textContent = "未验证";
  byId("metric-backtest").textContent = "Backtest 引用未知";
  byId("metric-health").textContent = "未知";
  byId("metric-freshness").textContent = "时效未知";
  byId("backtest-summary").textContent = "等待历史引用";
  byId("demo-summary").textContent = "等待 Sandbox 状态";
  byId("run-table-body").innerHTML = `<tr><td colspan="6" class="empty">${escapeHtml(message)}</td></tr>`;
  byId("axis-list").innerHTML = `<p class="empty">${escapeHtml(message)}</p>`;
  byId("inspector-title").textContent = "Run 未加载";
  byId("inspector-kv").innerHTML = `<p class="empty">${escapeHtml(message)}</p>`;
  byId("boundary-list").innerHTML = '<p class="empty">等待验证</p>';
  byId("source-list").innerHTML = '<p class="empty">等待验证</p>';
  byId("dock-content").innerHTML = `<p class="empty">${escapeHtml(message)}</p>`;
  byId("status-data").textContent = "数据：未知";
  byId("status-account").textContent = "账户：未知";
  byId("status-venue").textContent = "Venue：未知";
  byId("status-node").textContent = "节点：未知";
  byId("status-updated").textContent = "更新：未知";
}

function renderDock() {
  if (!currentProjection) return;
  const { business } = currentProjection;
  const cards = {
    positions: ["持仓", business.positions, "当前持仓投影", "统一读模型只读摘要"],
    activity: ["活动", business.lifecycle, "当前运行活动", "节点生命周期只读摘要"],
    fills: ["成交", business.fills, "当前成交投影", "不提供订单操作"],
    logs: ["日志", { summary: business.diagnostic, source_ref: business.source_ref }, "当前诊断摘要", "原始日志不在主产品面暴露"],
  };
  const [title, component, subtitle, boundary] = cards[currentDock];
  byId("dock-content").innerHTML = `<div class="dock-summary"><article><span>${title}</span><strong>${escapeHtml(dashboardValue(component?.summary))}</strong><small>${subtitle}</small></article><article><span>状态</span><strong>${escapeHtml(label(dashboardValue(component?.status)))}</strong><small>${escapeHtml(dashboardValue(component?.freshness_status))}</small></article><article><span>产品边界</span><strong>只读</strong><small>${boundary}</small></article></div>`;
}

function renderProjection(projection) {
  currentProjection = projection;
  const { identities, status, business, boundaries, sourceRefs, generatedAt } = projection;
  byId("strategy-name").textContent = identities.strategy_id;
  byId("strategy-version").textContent = `${identities.strategy_version} · 已冻结`;
  byId("environment").textContent = label(identities.environment);
  byId("account-id").textContent = identities.account_id;
  byId("venue-id").textContent = identities.venue_id;
  byId("run-title").textContent = `${identities.strategy_id} · ${identities.strategy_version}`;
  byId("metric-run").textContent = identities.strategy_instance_id;
  byId("metric-runtime").textContent = label(status.runtime.status);
  byId("metric-research").textContent = label(status.research.status);
  byId("metric-backtest").textContent = identities.backtest_run_id;
  byId("metric-health").textContent = label(status.technical_health.status);
  byId("metric-freshness").textContent = `时效 ${label(status.technical_health.freshness)}`;
  byId("backtest-summary").textContent = `已绑定 ${identities.backtest_run_id}`;
  byId("demo-summary").textContent = `${label(status.runtime.status)} · ${identities.node_id}`;
  byId("demo-mode-state").textContent = label(status.runtime.status);
  byId("run-table-body").innerHTML = `<tr><td>${escapeHtml(identities.strategy_instance_id)}</td><td>Demo</td><td>${escapeHtml(label(status.runtime.status))}</td><td>${escapeHtml(identities.account_id)}</td><td>${escapeHtml(identities.venue_id)}</td><td>${escapeHtml(dashboardValue(business.source_ref))}</td></tr>`;
  const axes = [["研究引用", status.research], ["运行状态", status.runtime], ["技术健康", status.technical_health], ["交易准备度", status.trading_readiness]];
  byId("axis-list").innerHTML = axes.map(([name, axis]) => `<div class="axis-item"><div><strong>${name}</strong><small>${escapeHtml(label(axis.freshness))} · ${escapeHtml(axis.source_refs[0])}</small></div><span class="${axis.status === "blocked" ? "blocked" : ""}">${escapeHtml(label(axis.status))}</span></div>`).join("");
  byId("inspector-title").textContent = identities.strategy_instance_id;
  byId("inspector-kv").innerHTML = [["策略", identities.strategy_id], ["版本", identities.strategy_version], ["Environment", identities.environment], ["账户", identities.account_id], ["Venue", identities.venue_id], ["节点", identities.node_id]].map(([key, value]) => `<div class="kv-row"><span>${key}</span><strong>${escapeHtml(value)}</strong></div>`).join("");
  byId("boundary-list").innerHTML = `<div class="boundary-item"><strong>共享只读</strong>页面只消费版本化状态投影</div><div class="boundary-item"><strong>真实交易关闭</strong>连接、订单写入与自动操作全部为 ${boundaries.real_orders_submitted}</div><div class="boundary-item"><strong>Live 独立准入</strong>Demo 状态不能推导 Live 权限</div>`;
  byId("source-list").innerHTML = sourceRefs.map((source) => `<div class="source-item">${escapeHtml(source)}</div>`).join("");
  const updated = new Date(generatedAt).toLocaleTimeString("zh-CN", { hour12: false });
  byId("status-data").textContent = `数据：${escapeHtml(dashboardValue(business.freshness_status))}`;
  byId("status-account").textContent = `账户：${identities.account_id}`;
  byId("status-venue").textContent = `Venue：${identities.venue_id}`;
  byId("status-node").textContent = `节点：${identities.node_id}`;
  byId("status-updated").textContent = `更新：${updated}`;
  renderDock();
}

function renderReady() {
  byId("connection-banner").className = "connection-banner ready";
  byId("connection-title").textContent = "策略状态已验证";
  byId("connection-detail").textContent = "当前使用单节点 Demo 共享读模型；Live 产品能力未开放";
  byId("connection-state").textContent = "只读在线";
  byId("top-health").className = "health-chip ready";
  byId("top-health").textContent = "共享状态在线";
  byId("rail-dot").className = "status-dot ready";
  byId("rail-state").textContent = "Demo 在线";
}
function renderBlocked(error) {
  byId("connection-banner").className = "connection-banner blocked";
  byId("connection-title").textContent = "策略工作台已阻断";
  byId("connection-detail").textContent = error instanceof Error ? error.message : "共享状态不可用";
  byId("connection-state").textContent = "Fail closed";
  byId("top-health").className = "health-chip blocked";
  byId("top-health").textContent = "状态不可用";
  byId("rail-dot").className = "status-dot blocked";
  byId("rail-state").textContent = "状态阻断";
}
async function refreshStrategyWorkbench() {
  byId("refresh").disabled = true;
  resetSurface("刷新中，旧状态已清空");
  try {
    const response = await fetch(SHARED_STATUS_URL, { method: "GET", cache: "no-store", headers: { Accept: "application/json" } });
    if (!response.ok) throw new Error(`共享状态 HTTP ${response.status}`);
    renderProjection(validateSharedStatus(await response.json()));
    renderReady();
  } catch (error) {
    resetSurface("共享状态不可用");
    renderBlocked(error);
  } finally {
    byId("refresh").disabled = false;
  }
}

byId("refresh").addEventListener("click", refreshStrategyWorkbench);
byId("drawer-toggle").addEventListener("click", () => {
  const shell = byId("strategy-workbench");
  const open = shell.classList.toggle("drawer-open");
  byId("drawer-toggle").textContent = open ? "›" : "‹";
  byId("drawer-toggle").setAttribute("aria-expanded", String(open));
  byId("drawer-toggle").setAttribute("aria-label", open ? "收起详情栏" : "展开详情栏");
  byId("drawer-toggle").title = open ? "收起详情栏" : "展开详情栏";
});
document.querySelectorAll(".dock-tabs button").forEach((button) => button.addEventListener("click", () => {
  document.querySelectorAll(".dock-tabs button").forEach((item) => { item.classList.remove("active"); item.setAttribute("aria-selected", "false"); });
  button.classList.add("active");
  button.setAttribute("aria-selected", "true");
  currentDock = button.dataset.dock;
  renderDock();
}));
document.querySelectorAll(".nav-item[data-section]").forEach((button) => button.addEventListener("click", () => {
  document.querySelectorAll(".nav-item").forEach((item) => item.classList.remove("active"));
  button.classList.add("active");
  byId("section-label").textContent = button.dataset.section;
}));
document.querySelectorAll(".mode-tabs button[data-mode]").forEach((button) => button.addEventListener("click", () => {
  document.querySelectorAll(".mode-tabs button").forEach((item) => item.classList.remove("active"));
  button.classList.add("active");
}));

if (typeof window !== "undefined" && window.matchMedia("(max-width: 760px)").matches) {
  byId("strategy-workbench").classList.remove("drawer-open");
  byId("drawer-toggle").textContent = "‹";
  byId("drawer-toggle").setAttribute("aria-expanded", "false");
  byId("drawer-toggle").setAttribute("aria-label", "展开详情栏");
  byId("drawer-toggle").title = "展开详情栏";
}
resetSurface();
refreshStrategyWorkbench();
"#;
