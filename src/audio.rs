//! Procedurally synthesised sound effects, rendered to in-memory WAV files at
//! start-up. The game queues `Sfx` values; the platform layer plays them.

use crate::util::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Sfx {
    Step,
    Swing,
    Hit,
    Miss,
    PlayerHurt,
    MonsterHurt,
    MonsterDie,
    DoorOpen,
    DoorClose,
    Locked,
    Pickup,
    Drop,
    Splash,
    Spell,
    Fizzle,
    Eat,
    Drink,
    Click,
    LevelUp,
    Thud,
    Fire,
    Zap,
    Lever,
    Growl,
}

pub const ALL_SFX: [Sfx; 24] = [
    Sfx::Step,
    Sfx::Swing,
    Sfx::Hit,
    Sfx::Miss,
    Sfx::PlayerHurt,
    Sfx::MonsterHurt,
    Sfx::MonsterDie,
    Sfx::DoorOpen,
    Sfx::DoorClose,
    Sfx::Locked,
    Sfx::Pickup,
    Sfx::Drop,
    Sfx::Splash,
    Sfx::Spell,
    Sfx::Fizzle,
    Sfx::Eat,
    Sfx::Drink,
    Sfx::Click,
    Sfx::LevelUp,
    Sfx::Thud,
    Sfx::Fire,
    Sfx::Zap,
    Sfx::Lever,
    Sfx::Growl,
];

const RATE: f32 = 22050.0;

struct Synth {
    buf: Vec<f32>,
    rng: Rng,
}

impl Synth {
    fn new(seconds: f32, seed: u64) -> Synth {
        Synth { buf: vec![0.0; (seconds * RATE) as usize], rng: Rng::new(seed) }
    }
    /// Filtered noise burst. `cut` 0..1 is a one-pole low-pass coefficient.
    fn noise(&mut self, start: f32, dur: f32, amp: f32, cut0: f32, cut1: f32, attack: f32) {
        let s = (start * RATE) as usize;
        let n = (dur * RATE) as usize;
        let mut lp = 0.0f32;
        for i in 0..n {
            if s + i >= self.buf.len() {
                break;
            }
            let t = i as f32 / n as f32;
            let cut = cut0 + (cut1 - cut0) * t;
            let w = self.rng.f32() * 2.0 - 1.0;
            lp += (w - lp) * cut;
            let env = env(t, attack);
            self.buf[s + i] += lp * amp * env;
        }
    }
    /// Pitched tone with a frequency sweep; `shape` 0 = sine, 1 = square, 2 = saw.
    #[allow(clippy::too_many_arguments)]
    fn tone(&mut self, start: f32, dur: f32, f0: f32, f1: f32, amp: f32, shape: u8, attack: f32, vibrato: f32) {
        let s = (start * RATE) as usize;
        let n = (dur * RATE) as usize;
        let mut ph = 0.0f32;
        for i in 0..n {
            if s + i >= self.buf.len() {
                break;
            }
            let t = i as f32 / n as f32;
            let f = f0 * (f1 / f0).powf(t) * (1.0 + (i as f32 / RATE * 30.0).sin() * vibrato);
            ph += f / RATE;
            let x = match shape {
                0 => (ph * std::f32::consts::TAU).sin(),
                1 => {
                    if ph.fract() < 0.5 {
                        0.6
                    } else {
                        -0.6
                    }
                }
                _ => (ph.fract() * 2.0 - 1.0) * 0.7,
            };
            self.buf[s + i] += x * amp * env(t, attack);
        }
    }
    fn wav(&self) -> Vec<u8> {
        let n = self.buf.len();
        let mut out = Vec::with_capacity(44 + n * 2);
        let data_len = (n * 2) as u32;
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data_len).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes()); // PCM
        out.extend_from_slice(&1u16.to_le_bytes()); // mono
        out.extend_from_slice(&(RATE as u32).to_le_bytes());
        out.extend_from_slice(&((RATE as u32) * 2).to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        for &v in &self.buf {
            let s = (v.clamp(-1.0, 1.0) * 30000.0) as i16;
            out.extend_from_slice(&s.to_le_bytes());
        }
        out
    }
}

fn env(t: f32, attack: f32) -> f32 {
    if t < attack {
        t / attack.max(1e-4)
    } else {
        let d = (t - attack) / (1.0 - attack).max(1e-4);
        (1.0 - d).max(0.0).powf(1.6)
    }
}

pub fn synth(s: Sfx) -> Vec<u8> {
    let seed = s as u64 + 1;
    let mut y;
    match s {
        Sfx::Step => {
            y = Synth::new(0.12, seed);
            y.noise(0.0, 0.1, 0.5, 0.25, 0.05, 0.05);
            y.tone(0.0, 0.06, 90.0, 60.0, 0.3, 0, 0.05, 0.0);
        }
        Sfx::Swing => {
            y = Synth::new(0.3, seed);
            y.noise(0.0, 0.28, 0.5, 0.05, 0.5, 0.5);
        }
        Sfx::Hit => {
            y = Synth::new(0.25, seed);
            y.noise(0.0, 0.2, 0.9, 0.6, 0.1, 0.02);
            y.tone(0.0, 0.15, 160.0, 70.0, 0.5, 0, 0.02, 0.0);
        }
        Sfx::Miss => {
            y = Synth::new(0.25, seed);
            y.noise(0.0, 0.22, 0.35, 0.08, 0.3, 0.4);
        }
        Sfx::PlayerHurt => {
            y = Synth::new(0.35, seed);
            y.tone(0.0, 0.3, 220.0, 120.0, 0.45, 2, 0.05, 0.03);
            y.noise(0.0, 0.1, 0.4, 0.5, 0.1, 0.05);
        }
        Sfx::MonsterHurt => {
            y = Synth::new(0.3, seed);
            y.tone(0.0, 0.25, 300.0, 180.0, 0.4, 2, 0.05, 0.05);
        }
        Sfx::MonsterDie => {
            y = Synth::new(0.7, seed);
            y.tone(0.0, 0.65, 260.0, 60.0, 0.45, 2, 0.05, 0.06);
            y.noise(0.2, 0.4, 0.3, 0.2, 0.05, 0.1);
        }
        Sfx::DoorOpen => {
            y = Synth::new(0.7, seed);
            y.tone(0.0, 0.5, 300.0, 520.0, 0.18, 2, 0.2, 0.08);
            y.noise(0.45, 0.2, 0.5, 0.3, 0.05, 0.02);
        }
        Sfx::DoorClose => {
            y = Synth::new(0.4, seed);
            y.noise(0.0, 0.3, 0.9, 0.4, 0.05, 0.01);
            y.tone(0.0, 0.2, 110.0, 70.0, 0.5, 0, 0.01, 0.0);
        }
        Sfx::Locked => {
            y = Synth::new(0.25, seed);
            y.noise(0.0, 0.05, 0.6, 0.9, 0.5, 0.01);
            y.noise(0.1, 0.05, 0.6, 0.9, 0.5, 0.01);
        }
        Sfx::Pickup => {
            y = Synth::new(0.12, seed);
            y.tone(0.0, 0.1, 500.0, 900.0, 0.25, 0, 0.05, 0.0);
        }
        Sfx::Drop => {
            y = Synth::new(0.15, seed);
            y.noise(0.0, 0.12, 0.6, 0.4, 0.05, 0.02);
        }
        Sfx::Splash => {
            y = Synth::new(0.5, seed);
            y.noise(0.0, 0.45, 0.6, 0.7, 0.1, 0.05);
        }
        Sfx::Spell => {
            y = Synth::new(0.8, seed);
            y.tone(0.0, 0.75, 300.0, 1200.0, 0.2, 0, 0.1, 0.02);
            y.tone(0.05, 0.7, 450.0, 1800.0, 0.12, 0, 0.1, 0.02);
        }
        Sfx::Fizzle => {
            y = Synth::new(0.5, seed);
            y.noise(0.0, 0.45, 0.4, 0.9, 0.2, 0.05);
            y.tone(0.0, 0.3, 400.0, 100.0, 0.15, 0, 0.05, 0.0);
        }
        Sfx::Eat => {
            y = Synth::new(0.4, seed);
            for k in 0..3 {
                y.noise(k as f32 * 0.12, 0.07, 0.5, 0.5, 0.2, 0.05);
            }
        }
        Sfx::Drink => {
            y = Synth::new(0.5, seed);
            for k in 0..4 {
                y.tone(k as f32 * 0.11, 0.08, 300.0, 200.0, 0.3, 0, 0.1, 0.0);
            }
        }
        Sfx::Click => {
            y = Synth::new(0.05, seed);
            y.noise(0.0, 0.03, 0.4, 0.9, 0.9, 0.01);
        }
        Sfx::LevelUp => {
            y = Synth::new(1.0, seed);
            for (k, f) in [392.0, 523.0, 659.0, 784.0].iter().enumerate() {
                y.tone(k as f32 * 0.15, 0.5, *f, *f, 0.18, 1, 0.05, 0.0);
            }
        }
        Sfx::Thud => {
            y = Synth::new(0.3, seed);
            y.tone(0.0, 0.25, 80.0, 40.0, 0.7, 0, 0.01, 0.0);
            y.noise(0.0, 0.1, 0.4, 0.3, 0.05, 0.02);
        }
        Sfx::Fire => {
            y = Synth::new(0.8, seed);
            y.noise(0.0, 0.75, 0.6, 0.15, 0.6, 0.3);
        }
        Sfx::Zap => {
            y = Synth::new(0.5, seed);
            y.tone(0.0, 0.45, 1600.0, 200.0, 0.2, 1, 0.01, 0.2);
            y.noise(0.0, 0.3, 0.3, 0.9, 0.9, 0.01);
        }
        Sfx::Lever => {
            y = Synth::new(0.3, seed);
            y.noise(0.0, 0.08, 0.6, 0.8, 0.3, 0.01);
            y.noise(0.15, 0.1, 0.6, 0.6, 0.2, 0.01);
        }
        Sfx::Growl => {
            y = Synth::new(0.7, seed);
            y.tone(0.0, 0.65, 110.0, 80.0, 0.35, 2, 0.15, 0.15);
            y.noise(0.0, 0.6, 0.15, 0.1, 0.05, 0.2);
        }
    }
    y.wav()
}

/// Loaded sounds (window mode only).
pub struct SoundBank {
    sounds: Vec<(Sfx, macroquad::audio::Sound)>,
    music: Vec<macroquad::audio::Sound>,
    playing: Option<u8>,
}

impl SoundBank {
    pub async fn load() -> SoundBank {
        let mut sounds = Vec::new();
        for s in ALL_SFX {
            let bytes = synth(s);
            if let Ok(snd) = macroquad::audio::load_sound_from_bytes(&bytes).await {
                sounds.push((s, snd));
            }
        }
        let mut music = Vec::new();
        for t in 0..2u8 {
            if let Ok(snd) = macroquad::audio::load_sound_from_bytes(&self::music(t)).await {
                music.push(snd);
            }
        }
        SoundBank { sounds, music, playing: None }
    }

    /// Switch the looping music track (None = silence).
    pub fn set_music(&mut self, track: Option<u8>) {
        if self.playing == track {
            return;
        }
        if let Some(p) = self.playing {
            if let Some(s) = self.music.get(p as usize) {
                macroquad::audio::stop_sound(s);
            }
        }
        if let Some(t) = track {
            if let Some(s) = self.music.get(t as usize) {
                macroquad::audio::play_sound(s, macroquad::audio::PlaySoundParams { looped: true, volume: 0.32 });
            }
        }
        self.playing = track;
    }

    pub fn play(&self, s: Sfx) {
        if let Some((_, snd)) = self.sounds.iter().find(|(k, _)| *k == s) {
            macroquad::audio::play_sound(snd, macroquad::audio::PlaySoundParams { looped: false, volume: 0.45 });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn music_renders() {
        for t in 0..2 {
            let w = music(t);
            assert_eq!(&w[0..4], b"RIFF");
            assert!(w.len() > 22050 * 2 * 20, "track {} is too short", t);
        }
    }

    #[test]
    fn all_sounds_are_valid_wavs() {
        for s in ALL_SFX {
            let w = synth(s);
            assert_eq!(&w[0..4], b"RIFF");
            assert!(w.len() > 44 + 100);
        }
    }
}

// ------------------------------------------------------------------ music

/// Equal-tempered frequency of a MIDI note.
fn midi(n: i32) -> f32 {
    440.0 * 2f32.powf((n - 69) as f32 / 12.0)
}

struct Mixer {
    buf: Vec<f32>,
}

impl Mixer {
    fn add_note(&mut self, start: f32, dur: f32, freq: f32, amp: f32, inst: u8) {
        let s0 = (start * RATE) as usize;
        let release = match inst {
            0 => 1.2,
            1 => 0.4,
            _ => 0.25,
        };
        let n = ((dur + release) * RATE) as usize;
        let mut ph = 0.0f32;
        for i in 0..n {
            let idx = s0 + i;
            if idx >= self.buf.len() {
                break;
            }
            let t = i as f32 / RATE;
            let env = match inst {
                // pad: slow attack, sustain, release
                0 => {
                    let a = (t / 0.9).min(1.0);
                    let r = if t > dur { (1.0 - (t - dur) / release).max(0.0) } else { 1.0 };
                    a * r
                }
                // bass: pluck
                1 => {
                    let a = (t / 0.02).min(1.0);
                    let d = (-t * 2.2).exp();
                    let r = if t > dur { (1.0 - (t - dur) / release).max(0.0) } else { 1.0 };
                    a * (0.35 + 0.65 * d) * r
                }
                // lead: flute-like
                _ => {
                    let a = (t / 0.06).min(1.0);
                    let s = 0.75 + 0.25 * (-t * 3.0).exp();
                    let r = if t > dur { (1.0 - (t - dur) / release).max(0.0) } else { 1.0 };
                    a * s * r
                }
            };
            let vib = if inst == 2 { 1.0 + (t * 5.2 * std::f32::consts::TAU).sin() * 0.004 * (t / 0.4).min(1.0) } else { 1.0 };
            ph += freq * vib / RATE;
            let x = ph * std::f32::consts::TAU;
            let w = match inst {
                0 => x.sin() * 0.6 + (x * 2.0).sin() * 0.25 + (x * 3.0).sin() * 0.12 + (x * 1.003).sin() * 0.3,
                1 => x.sin() * 0.8 + (x * 2.0).sin() * 0.3,
                _ => x.sin() * 0.85 + (x * 2.0).sin() * 0.12 + (x * 3.0).sin() * 0.05,
            };
            self.buf[idx] += w * amp * env;
        }
    }

    fn reverb(&mut self) {
        let taps = [(0.113f32, 0.32f32), (0.197, 0.26), (0.281, 0.2), (0.379, 0.14)];
        let src = self.buf.clone();
        for (delay, gain) in taps {
            let d = (delay * RATE) as usize;
            for i in d..self.buf.len() {
                self.buf[i] += src[i - d] * gain + self.buf[i - d] * gain * 0.35;
            }
        }
    }

    fn normalize(&mut self, peak: f32) {
        let m = self.buf.iter().fold(0.0f32, |a, &v| a.max(v.abs())).max(1e-6);
        let k = peak / m;
        for v in self.buf.iter_mut() {
            *v *= k;
        }
    }
}

/// Compose and render a looping music track. 0 = title, 1 = dungeon.
pub fn music(track: u8) -> Vec<u8> {
    // Chords as (root midi note, minor?) per bar.
    let (bpm, chords, seed): (f32, &[(i32, bool)], u64) = if track == 0 {
        (64.0, &[(50, true), (48, false), (46, false), (45, false), (50, true), (53, false), (48, false), (45, false)], 17)
    } else {
        (
            70.0,
            &[(50, true), (50, true), (46, false), (48, false), (50, true), (43, true), (45, false), (45, false), (50, true), (41, false), (48, false), (50, true), (46, false), (43, true), (45, false), (50, true)],
            29,
        )
    };
    let beat = 60.0 / bpm;
    let bar = beat * 4.0;
    let total = bar * chords.len() as f32;
    let mut mx = Mixer { buf: vec![0.0; ((total + 0.1) * RATE) as usize] };
    let mut rng = Rng::new(seed);
    // D natural minor scale (with C sharp over A chords).
    let scale: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];
    let mut melody_note: i32 = 62;
    for (bi, &(root, minor)) in chords.iter().enumerate() {
        let t0 = bi as f32 * bar;
        let third = if minor { 3 } else { 4 };
        // Pad chord (root position, an octave up), with a slight overlap.
        for iv in [0, third, 7] {
            mx.add_note(t0, bar * 0.98, midi(root + 12 + iv), 0.07, 0);
        }
        // Bass on beats 1 and 3.
        mx.add_note(t0, beat * 1.8, midi(root - 12), 0.16, 1);
        mx.add_note(t0 + beat * 2.0, beat * 1.8, midi(root - 12 + if rng.chance(0.5) { 7 } else { 0 }), 0.12, 1);
        // Melody: a gentle walk over the scale, landing on chord tones.
        let sparse = track == 1 && bi % 4 == 3;
        let mut t = t0;
        while t < t0 + bar - 0.01 && !sparse {
            let durs = [beat * 2.0, beat, beat, beat * 0.5, beat * 1.5];
            let mut d = *rng.pick(&durs);
            if t + d > t0 + bar {
                d = t0 + bar - t;
            }
            if rng.chance(0.15) {
                t += d;
                continue;
            }
            let step = rng.range(-2, 3);
            let deg = scale.iter().position(|&s| (melody_note - 50).rem_euclid(12) == s).unwrap_or(0) as i32;
            let nd = (deg + step).rem_euclid(7);
            let oct = (melody_note - 50).div_euclid(12) + if deg + step >= 7 { 1 } else if deg + step < 0 { -1 } else { 0 };
            let mut note = 50 + oct * 12 + scale[nd as usize];
            // Leading tone over A major chords.
            if root % 12 == 9 && !minor && (note - 50).rem_euclid(12) == 0 {
                note -= 1;
            }
            note = note.clamp(60, 79);
            // On strong beats prefer chord tones.
            let pos = ((t - t0) / beat).round() as i32;
            if pos % 2 == 0 {
                let tones = [root + 12, root + 12 + third, root + 19, root + 24];
                note = *tones.iter().min_by_key(|&&c| (c - note).abs()).unwrap();
                note = note.clamp(60, 79);
            }
            melody_note = note;
            mx.add_note(t, d * 0.92, midi(note), if track == 0 { 0.12 } else { 0.09 }, 2);
            t += d;
        }
    }
    mx.reverb();
    // Fold the reverb tail back to the start so the loop is seamless.
    mx.normalize(0.55);
    let synth = Synth { buf: mx.buf, rng: Rng::new(1) };
    synth.wav()
}
