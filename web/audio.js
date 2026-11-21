/**
 * WebAudio synthesizer for retro Lunar Lander sound effects:
 * continuous filtered-noise thruster, low-fuel beep, landing chime, and explosion.
 */
class RetroAudio {
  constructor() {
    this.ctx = null;
    this.thrustGain = null;
  }

  /** Lazily initializes AudioContext on first user gesture. */
  ensureContext() {
    if (this.ctx) {
      if (this.ctx.state === "suspended") this.ctx.resume();
      return;
    }
    const AudioCtx = window.AudioContext || window.webkitAudioContext;
    if (!AudioCtx) return;
    this.ctx = new AudioCtx();

    // Continuous white-noise thruster through a low-pass filter
    const bufferSize = this.ctx.sampleRate * 2;
    const noiseBuffer = this.ctx.createBuffer(1, bufferSize, this.ctx.sampleRate);
    const data = noiseBuffer.getChannelData(0);
    for (let i = 0; i < bufferSize; i++) {
      data[i] = Math.random() * 2 - 1;
    }

    const noise = this.ctx.createBufferSource();
    noise.buffer = noiseBuffer;
    noise.loop = true;

    const filter = this.ctx.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.value = 260;

    this.thrustGain = this.ctx.createGain();
    this.thrustGain.gain.value = 0;

    noise.connect(filter);
    filter.connect(this.thrustGain);
    this.thrustGain.connect(this.ctx.destination);
    noise.start();
  }

  /** Sets continuous thruster volume in [0, 1]. */
  setThrust(level) {
    if (!this.thrustGain || !this.ctx) return;
    this.thrustGain.gain.setTargetAtTime(level * 0.22, this.ctx.currentTime, 0.03);
  }

  /** Plays a one-shot sound effect (1 = beep, 2 = explosion, 3 = landing). */
  playEvent(eventId) {
    if (!this.ctx) return;
    const now = this.ctx.currentTime;
    if (eventId === 1) {
      this.tone(880, 0.08, "square", 0.08, now);
    } else if (eventId === 2) {
      this.explosion(now);
    } else if (eventId === 3) {
      this.tone(523.25, 0.12, "triangle", 0.12, now);
      this.tone(659.25, 0.18, "triangle", 0.12, now + 0.12);
    }
  }

  tone(freq, dur, type, vol, start) {
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = type;
    osc.frequency.setValueAtTime(freq, start);
    gain.gain.setValueAtTime(vol, start);
    gain.gain.exponentialRampToValueAtTime(0.001, start + dur);
    osc.connect(gain);
    gain.connect(this.ctx.destination);
    osc.start(start);
    osc.stop(start + dur);
  }

  explosion(start) {
    const dur = 0.7;
    const buf = this.ctx.createBuffer(1, this.ctx.sampleRate * dur, this.ctx.sampleRate);
    const out = buf.getChannelData(0);
    for (let i = 0; i < out.length; i++) {
      out[i] = (Math.random() * 2 - 1) * (1 - i / out.length);
    }
    const src = this.ctx.createBufferSource();
    src.buffer = buf;
    const gain = this.ctx.createGain();
    gain.gain.setValueAtTime(0.35, start);
    gain.gain.exponentialRampToValueAtTime(0.01, start + dur);
    src.connect(gain);
    gain.connect(this.ctx.destination);
    src.start(start);
  }
}

window.retroAudio = new RetroAudio();
