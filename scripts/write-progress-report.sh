#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PROGRESS_JSON="${PROGRESS_JSON:-${ROOT_DIR}/docs/aqua-linux/progress.json}"
PROGRESS_HTML="${PROGRESS_HTML:-${ROOT_DIR}/docs/aqua-linux/progress.html}"

if [ ! -f "${PROGRESS_JSON}" ]; then
    echo "Missing progress JSON: ${PROGRESS_JSON}" >&2
    exit 1
fi

export PROGRESS_JSON PROGRESS_HTML

python3 - <<'PY'
import html
import json
import os

source = os.environ["PROGRESS_JSON"]
target = os.environ["PROGRESS_HTML"]

with open(source, "r", encoding="utf-8") as handle:
    data = json.load(handle)


def esc(value):
    return html.escape(str(value), quote=True)


def status_label(status):
    return status.replace("-", " ").title()


def phase_sort_key(phase):
    return (phase.get("updated", "0000-00-00"), phase.get("id", ""))


phases = sorted(data["phases"], key=phase_sort_key, reverse=True)
phase_rows = []
for phase in phases:
    percent = int(phase["percent"])
    status = esc(phase["status"])
    phase_rows.append(
        f"""
              <tr class="{status}">
                <td class="col-id">{esc(phase['id']).upper()}</td>
                <td class="col-name">
                  <strong>{esc(phase['name'])}</strong>
                  <span>{esc(phase['summary'])}</span>
                </td>
                <td class="col-date">{esc(phase['updated'])}</td>
                <td class="col-status">{status_label(phase['status'])}</td>
                <td class="col-progress">
                  <div class="progress" aria-label="{esc(phase['name'])} progress">
                    <span style="width: {percent}%"></span>
                  </div>
                  <b>{percent}%</b>
                </td>
              </tr>
        """.strip()
    )

phase_rows_html = "\n".join(phase_rows)
rules = "\n".join(f"<li>{esc(item)}</li>" for item in data["rules"])
next_steps = "\n".join(f"<li>{esc(item)}</li>" for item in data["nextSteps"])
overall = int(data["overallPercent"])

document = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{esc(data['product'])} {esc(data['release'])} Progress</title>
  <style>
    :root {{
      color-scheme: light;
      --ink: #1d2329;
      --muted: #66717b;
      --aqua: #008fc4;
      --aqua-strong: #006f9e;
      --green: #29723c;
      --line: #9da7b0;
      --line-dark: #6f7983;
      --sidebar: #d8e1e8;
      --row: #ffffff;
      --row-alt: #eef1f4;
      --selected: #bfd9ec;
      --shadow: rgba(22, 28, 34, 0.34);
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      min-height: 100vh;
      font-family: "Lucida Grande", "Helvetica Neue", Helvetica, Arial, sans-serif;
      letter-spacing: 0;
      color: var(--ink);
      background: #cfd4da;
    }}
    main {{
      width: min(1220px, calc(100vw - 28px));
      margin: 0 auto;
      padding: 34px 0;
    }}
    .window {{
      overflow: hidden;
      border: 1px solid #5f6972;
      border-radius: 8px;
      background: #f7f8f9;
      box-shadow:
        0 22px 56px var(--shadow),
        inset 0 1px 0 rgba(255, 255, 255, 0.92);
    }}
    .titlebar {{
      display: grid;
      grid-template-columns: auto 1fr auto;
      align-items: center;
      gap: 12px;
      min-height: 38px;
      padding: 7px 12px;
      border-bottom: 1px solid #8d969e;
      background: linear-gradient(#f5f6f7, #c9cfd5);
    }}
    .lights {{
      display: flex;
      gap: 7px;
    }}
    .lights span {{
      width: 13px;
      height: 13px;
      border-radius: 50%;
      border: 1px solid #777f86;
      background: linear-gradient(#ffffff, #aeb7bf);
      box-shadow: inset 0 1px 0 rgba(255,255,255,0.9);
    }}
    .title {{
      text-align: center;
      font-size: 13px;
      font-weight: 800;
      text-shadow: 0 1px 0 #fff;
    }}
    .search {{
      width: 190px;
      height: 23px;
      border: 1px solid #8d969e;
      border-radius: 12px;
      background: #fff;
      box-shadow: inset 0 1px 2px rgba(0,0,0,0.15), 0 1px 0 rgba(255,255,255,0.8);
    }}
    .toolbar {{
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
      padding: 11px 14px;
      border-bottom: 1px solid #a8b0b8;
      background: linear-gradient(#edf0f2, #cfd6dc);
    }}
    .toolbar h1 {{
      margin: 0;
      font-size: 22px;
      line-height: 1.1;
      color: #141a20;
      text-shadow: 0 1px 0 #fff;
    }}
    .toolbar p {{
      margin: 4px 0 0;
      color: #4a5661;
      font-size: 13px;
      line-height: 1.35;
    }}
    .meter {{
      display: grid;
      place-items: center;
      min-width: 118px;
      height: 58px;
      border: 1px solid #7d8791;
      border-radius: 7px;
      background: linear-gradient(#ffffff, #d9dee3 48%, #c4ccd4 49%, #f7f8f9);
      box-shadow: inset 0 1px 0 #fff;
    }}
    .meter strong {{
      color: var(--aqua-strong);
      font-size: 28px;
      line-height: 1;
    }}
    .meter span {{
      color: var(--muted);
      font-size: 10px;
      font-weight: 800;
      text-transform: uppercase;
    }}
    .content {{
      display: grid;
      grid-template-columns: 214px 1fr;
      min-height: 620px;
    }}
    aside {{
      border-right: 1px solid #9da7b0;
      background: var(--sidebar);
      padding: 14px 10px;
    }}
    .side-title {{
      margin: 0 0 7px;
      color: #5b6874;
      font-size: 11px;
      font-weight: 900;
      text-transform: uppercase;
      text-shadow: 0 1px 0 rgba(255,255,255,0.75);
    }}
    .side-row {{
      display: flex;
      justify-content: space-between;
      gap: 10px;
      padding: 5px 7px;
      border-radius: 5px;
      font-size: 12px;
      color: #1f2b35;
    }}
    .side-row.active {{
      color: #fff;
      background: linear-gradient(#6ca5d4, #337bb5);
      text-shadow: 0 -1px 0 rgba(0,0,0,0.25);
    }}
    .table-wrap {{
      overflow: auto;
      background: #fff;
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
      table-layout: fixed;
      font-size: 13px;
    }}
    thead th {{
      position: sticky;
      top: 0;
      z-index: 1;
      height: 27px;
      padding: 0 10px;
      color: #1c242c;
      text-align: left;
      font-weight: 800;
      border-right: 1px solid #aab3bc;
      border-bottom: 1px solid #8f99a2;
      background: linear-gradient(#f9fafb, #d4dae0);
      text-shadow: 0 1px 0 #fff;
    }}
    tbody tr:nth-child(odd) {{ background: var(--row); }}
    tbody tr:nth-child(even) {{ background: var(--row-alt); }}
    tbody tr.in-progress, tbody tr.early {{ background: #eef8fd; }}
    tbody tr.complete {{ color: #19251d; }}
    td {{
      padding: 9px 10px;
      border-right: 1px solid #d3d9df;
      border-bottom: 1px solid #e2e6ea;
      vertical-align: middle;
    }}
    .col-id {{ width: 72px; color: #41505d; font-weight: 900; }}
    .col-name {{ width: auto; }}
    .col-name strong {{
      display: block;
      color: #15202a;
      font-size: 14px;
    }}
    .col-name span {{
      display: block;
      margin-top: 3px;
      color: #53616d;
      line-height: 1.35;
    }}
    .col-date {{ width: 122px; white-space: nowrap; color: #2f3b45; }}
    .col-status {{
      width: 122px;
      white-space: nowrap;
      color: var(--muted);
      font-weight: 800;
    }}
    .complete .col-status {{ color: var(--green); }}
    .in-progress .col-status, .early .col-status {{ color: var(--aqua-strong); }}
    .col-progress {{
      width: 178px;
      display: grid;
      grid-template-columns: 1fr 44px;
      align-items: center;
      gap: 10px;
    }}
    .col-progress b {{
      text-align: right;
      color: #26323c;
    }}
    .progress {{
      height: 10px;
      border: 1px solid #87919b;
      border-radius: 999px;
      background: #c4cbd2;
      box-shadow: inset 0 1px 2px rgba(0,0,0,0.22), 0 1px 0 #fff;
      overflow: hidden;
    }}
    .progress span {{
      display: block;
      height: 100%;
      border-radius: inherit;
      background: linear-gradient(#76dbff, #0096cf 52%, #007db6);
      box-shadow: inset 0 1px 0 rgba(255,255,255,0.75);
    }}
    .lower {{
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 14px;
      padding: 14px;
      border-top: 1px solid #b5bdc4;
      background: #eef1f4;
    }}
    .panel {{
      min-height: 138px;
      border: 1px solid #9da7b0;
      border-radius: 7px;
      background: linear-gradient(#ffffff, #e7ebef);
      box-shadow: inset 0 1px 0 #fff;
      padding: 14px 16px;
    }}
    .panel h2 {{
      margin: 0 0 9px;
      font-size: 15px;
      color: #1e2831;
      text-shadow: 0 1px 0 #fff;
    }}
    ul {{
      margin: 0;
      padding-left: 18px;
      color: #46535e;
      line-height: 1.55;
      font-size: 12px;
    }}
    footer {{
      padding: 10px 14px 12px;
      border-top: 1px solid #b8c0c7;
      color: var(--muted);
      font-size: 12px;
      background: #e5e9ed;
    }}
    @media (max-width: 900px) {{
      main {{ padding: 18px 0; }}
      .content {{ grid-template-columns: 1fr; }}
      aside {{ display: none; }}
      .toolbar {{ align-items: flex-start; flex-direction: column; }}
      .meter {{ width: 100%; }}
      .lower {{ grid-template-columns: 1fr; }}
      .search {{ display: none; }}
      table {{ min-width: 820px; }}
    }}
    @media (prefers-reduced-motion: reduce) {{
      * {{ scroll-behavior: auto; }}
    }}
  </style>
</head>
<body>
  <main>
    <section class="window">
      <div class="titlebar">
        <div class="lights"><span></span><span></span><span></span></div>
        <div class="title">{esc(data['product'])} {esc(data['release'])} progress report</div>
        <div class="search"></div>
      </div>

      <div class="toolbar">
        <div>
          <h1>{esc(data['product'])}</h1>
          <p>{esc(data['currentStage'])}</p>
        </div>
        <div class="meter">
          <strong>{overall}%</strong>
          <span>complete</span>
        </div>
      </div>

      <div class="content">
        <aside aria-label="Project metadata">
          <p class="side-title">Project</p>
          <div class="side-row active"><span>Base</span><b>{esc(data['base'])}</b></div>
          <div class="side-row"><span>Graphics</span><b>Custom</b></div>
          <div class="side-row"><span>Target</span><b>QEMU</b></div>
          <div class="side-row"><span>Hardware</span><b>Later</b></div>
          <p class="side-title" style="margin-top:16px">Updated</p>
          <div class="side-row"><span>Date</span><b>{esc(data['updated'])}</b></div>
          <div class="side-row"><span>Release</span><b>{esc(data['release'])}</b></div>
        </aside>

        <section class="table-wrap" aria-label="Aqua Linux v1.0 phases sorted by update date">
          <table>
            <thead>
              <tr>
                <th class="col-id">ID</th>
                <th class="col-name">Phase</th>
                <th class="col-date">Updated</th>
                <th class="col-status">Status</th>
                <th class="col-progress">Progress</th>
              </tr>
            </thead>
            <tbody>
{phase_rows_html}
            </tbody>
          </table>
        </section>
      </div>

      <section class="lower">
        <div class="panel">
          <h2>Progress Rules</h2>
          <ul>{rules}</ul>
        </div>
        <div class="panel">
          <h2>Next Steps</h2>
          <ul>{next_steps}</ul>
        </div>
      </section>

      <footer>
        Generated from docs/aqua-linux/progress.json. Update the changed phase date, then run scripts/write-progress-report.sh.
      </footer>
    </section>
  </main>
</body>
</html>
"""

with open(target, "w", encoding="utf-8") as handle:
    handle.write(document)

print(f"Aqua Linux progress report written: {target}")
PY
