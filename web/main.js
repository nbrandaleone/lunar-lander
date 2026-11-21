/**
 * Browser host for the Lunar Lander Rust/WASM module.
 * Provides Canvas 2D FFI bindings, HUD updates, and keyboard/touch input.
 */
(async function () {
  const canvas = document.getElementById("game-canvas");
  const ctx = canvas.getContext("2d");
  const overlayEl = document.getElementById("overlay");

  const hud = {
    score: document.getElementById("hud-score"),
    time: document.getElementById("hud-time"),
    fuel: document.getElementById("hud-fuel"),
    alt: document.getElementById("hud-alt"),
    vx: document.getElementById("hud-vx"),
    vy: document.getElementById("hud-vy"),
  };

  const keys = { left: false, right: false, thrust: false };
  let wasm = null;

  function pad(num, width) {
    const sign = num < 0 ? "-" : "";
    return sign + String(Math.abs(Math.trunc(num))).padStart(width, "0");
  }

  function formatTime(ms) {
    const totalSec = Math.floor(ms / 1000);
    const min = Math.floor(totalSec / 60);
    const sec = String(totalSec % 60).padStart(2, "0");
    return `${min}:${sec}`;
  }

  function overlayMessage(code, points, fuelLost) {
    switch (code) {
      case 1: return "INSERT COINS\n\nCLICK OR PRESS SPACE TO PLAY\nARROW KEYS TO MOVE";
      case 2: return `CONGRATULATIONS\nA PERFECT LANDING\n${points} POINTS`;
      case 3: return `YOU LANDED HARD\nYOU ARE HOPELESSLY MAROONED\n${points} POINTS`;
      case 4: return `AUXILIARY FUEL TANKS DESTROYED\n${fuelLost} FUEL UNITS LOST\n\nYOU JUST DESTROYED A 100 MEGABUCK LANDER`;
      case 5: return `AUXILIARY FUEL TANKS DESTROYED\n${fuelLost} FUEL UNITS LOST\n\nDESTROYED`;
      case 6: return `AUXILIARY FUEL TANKS DESTROYED\n${fuelLost} FUEL UNITS LOST\n\nYOU CREATED A TWO MILE CRATER`;
      case 7: return "OUT OF FUEL\n\nGAME OVER\nCLICK OR PRESS SPACE TO PLAY";
      case 8: return "LOW ON FUEL";
      case 9: return "OUT OF FUEL";
      default: return "";
    }
  }

  const env = {
    canvas_clear: (w, h) => ctx.clearRect(0, 0, w, h),
    canvas_save: () => ctx.save(),
    canvas_restore: () => ctx.restore(),
    canvas_translate: (x, y) => ctx.translate(x, y),
    canvas_scale: (sx, sy) => ctx.scale(sx, sy),
    canvas_rotate: (rad) => ctx.rotate(rad),
    canvas_set_stroke: (r, g, b, lw) => {
      ctx.strokeStyle = `rgb(${r},${g},${b})`;
      ctx.lineWidth = lw;
      ctx.lineJoin = "bevel";
    },
    canvas_begin_path: () => ctx.beginPath(),
    canvas_move_to: (x, y) => ctx.moveTo(x, y),
    canvas_line_to: (x, y) => ctx.lineTo(x, y),
    canvas_close_path: () => ctx.closePath(),
    canvas_rect: (x, y, w, h) => ctx.rect(x, y, w, h),
    canvas_stroke: () => ctx.stroke(),
    canvas_draw_zone_label: (mult, sx, sy) => {
      ctx.save();
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.fillStyle = "#ffffff";
      ctx.font = '12px "Courier New", monospace';
      ctx.textAlign = "center";
      ctx.fillText(`${mult}x`, sx, sy + 12);
      ctx.restore();
    },
    hud_update: (score, timeMs, fuel, alt, vx, vy) => {
      hud.score.textContent = pad(score, 4);
      hud.time.textContent = formatTime(timeMs);
      hud.fuel.textContent = pad(fuel, 4);
      hud.alt.textContent = pad(alt, 4);
      hud.vx.textContent = pad(vx, 3);
      hud.vy.textContent = pad(vy, 3);
    },
    hud_set_overlay: (code, points, fuelLost) => {
      overlayEl.textContent = overlayMessage(code, points, fuelLost);
    },
    audio_set_thrust: (lvl) => window.retroAudio.setThrust(lvl),
    audio_play_event: (id) => window.retroAudio.playEvent(id),
  };

  function syncInput() {
    if (wasm) {
      wasm.set_input(keys.left ? 1 : 0, keys.right ? 1 : 0, keys.thrust ? 1 : 0);
    }
  }

  window.addEventListener("keydown", (e) => {
    window.retroAudio.ensureContext();
    if (e.key === "ArrowLeft" || e.key === "a" || e.key === "A") keys.left = true;
    if (e.key === "ArrowRight" || e.key === "d" || e.key === "D") keys.right = true;
    if (e.key === "ArrowUp" || e.key === "w" || e.key === "W") keys.thrust = true;
    if (e.key === " " && wasm) wasm.start_or_restart();
    syncInput();
  });

  window.addEventListener("keyup", (e) => {
    if (e.key === "ArrowLeft" || e.key === "a" || e.key === "A") keys.left = false;
    if (e.key === "ArrowRight" || e.key === "d" || e.key === "D") keys.right = false;
    if (e.key === "ArrowUp" || e.key === "w" || e.key === "W") keys.thrust = false;
    syncInput();
  });

  window.addEventListener("mousedown", () => {
    window.retroAudio.ensureContext();
    if (wasm) wasm.start_or_restart();
  });

  window.addEventListener("touchstart", () => {
    window.retroAudio.ensureContext();
    if (wasm) wasm.start_or_restart();
  });

  function resize() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    if (wasm) wasm.resize_viewport(canvas.width, canvas.height);
  }
  window.addEventListener("resize", resize);

  const resp = await fetch("lunar_lander.wasm");
  const bytes = await resp.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, { env });
  wasm = instance.exports;

  resize();
  wasm.init_game(canvas.width, canvas.height);

  // Fixed 60Hz accumulator loop
  let last = performance.now();
  let acc = 0;
  const STEP = 1000 / 60;

  function frame(now) {
    acc += Math.min(now - last, 100);
    last = now;
    while (acc >= STEP) {
      wasm.step_frame();
      acc -= STEP;
    }
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
})();
