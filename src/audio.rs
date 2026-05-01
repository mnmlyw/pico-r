use crate::memory::{self, Memory};

pub const SAMPLE_RATE: u32 = 22050;
const SAMPLES_PER_TICK: u32 = 183;
pub const NUM_CHANNELS: usize = 4;
const MAX_FREQ: f32 = 65.41 * 38.055;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Waveform {
    Triangle = 0,
    TiltedSaw = 1,
    Saw = 2,
    Square = 3,
    Pulse = 4,
    Organ = 5,
    Noise = 6,
    Phaser = 7,
}

impl Waveform {
    fn from(b: u8) -> Self {
        match b & 7 {
            0 => Waveform::Triangle,
            1 => Waveform::TiltedSaw,
            2 => Waveform::Saw,
            3 => Waveform::Square,
            4 => Waveform::Pulse,
            5 => Waveform::Organ,
            6 => Waveform::Noise,
            _ => Waveform::Phaser,
        }
    }
}

#[derive(Clone)]
pub struct Channel {
    pub sfx_id: i8,
    pub note_index: u8,
    pub sub_tick: f32,
    pub phase: f64,
    pub volume: f32,
    pub base_volume: f32,
    pub frequency: f64,
    pub base_frequency: f64,
    pub prev_frequency: f64,
    pub waveform: Waveform,
    pub effect: u8,
    pub custom: bool,
    pub note_progress: f32,
    pub finished: bool,
    pub noise_sample: f32,
    pub noise_prev_sample: f32,
    pub inst_sfx_id: u8,
    pub inst_note_index: u8,
    pub inst_sub_tick: f32,
    pub inst_phase: f64,
    pub prev_pitch: u8,
    pub prev_vol: u8,
    pub loop_released: bool,
}

impl Channel {
    pub const fn new() -> Self {
        Self {
            sfx_id: -1,
            note_index: 0,
            sub_tick: 0.0,
            phase: 0.0,
            volume: 0.0,
            base_volume: 0.0,
            frequency: 0.0,
            base_frequency: 0.0,
            prev_frequency: 0.0,
            waveform: Waveform::Triangle,
            effect: 0,
            custom: false,
            note_progress: 0.0,
            finished: true,
            noise_sample: 0.0,
            noise_prev_sample: 0.0,
            inst_sfx_id: 0,
            inst_note_index: 0,
            inst_sub_tick: 0.0,
            inst_phase: 0.0,
            prev_pitch: 0,
            prev_vol: 0,
            loop_released: false,
        }
    }
}

#[derive(Clone)]
pub struct MusicState {
    pub pattern: i16,
    pub tick: u32,
    pub channel_mask: u8,
    pub loop_back: i16,
    pub playing: bool,
    pub total_patterns: u32,
    pub fade_samples: u32,
    pub fade_progress: u32,
    pub fade_out: bool,
}

impl MusicState {
    pub const fn new() -> Self {
        Self {
            pattern: -1,
            tick: 0,
            channel_mask: 0xF,
            loop_back: -1,
            playing: false,
            total_patterns: 0,
            fade_samples: 0,
            fade_progress: 0,
            fade_out: false,
        }
    }
}

pub struct Audio {
    pub channels: [Channel; NUM_CHANNELS],
    pub music_state: MusicState,
    pub noise_seed: u32,
}

impl Audio {
    pub fn new() -> Self {
        Self {
            channels: [
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
            ],
            music_state: MusicState::new(),
            noise_seed: 1,
        }
    }

    pub fn reset(&mut self) {
        self.channels = [
            Channel::new(),
            Channel::new(),
            Channel::new(),
            Channel::new(),
        ];
        self.music_state = MusicState::new();
        self.noise_seed = 1;
    }

    pub fn play_sfx(&mut self, memory: &Memory, sfx_id: i32, channel_req: i32, offset: i32) {
        if sfx_id == -1 {
            if channel_req >= 0 && (channel_req as usize) < NUM_CHANNELS {
                self.stop_channel(channel_req as usize);
            } else {
                for i in 0..NUM_CHANNELS {
                    self.stop_channel(i);
                }
                self.music_state.pattern = -1;
                self.music_state.playing = false;
            }
            return;
        }
        if sfx_id == -2 {
            if channel_req >= 0 && (channel_req as usize) < NUM_CHANNELS {
                self.release_loop(channel_req as usize);
            } else {
                for i in 0..NUM_CHANNELS {
                    self.release_loop(i);
                }
            }
            return;
        }
        if sfx_id < 0 || sfx_id >= 64 {
            return;
        }

        // sfx(N, -2): release the loop of any channel currently playing
        // SFX N. Carts pair this with sfx(N) for fresh starts, and call
        // sfx(N, -2) alone to STOP a loop without starting a new one. Without
        // this the sound loops forever (e.g. mansion_bros' vacuum SFX).
        if channel_req == -2 {
            for i in 0..NUM_CHANNELS {
                if self.channels[i].sfx_id as i32 == sfx_id {
                    self.release_loop(i);
                }
            }
            return;
        }

        let ch: usize = if channel_req >= 0 && (channel_req as usize) < NUM_CHANNELS {
            channel_req as usize
        } else {
            let mut found_free = None;
            for i in 0..NUM_CHANNELS {
                if self.channels[i].finished {
                    found_free = Some(i);
                    break;
                }
            }
            found_free.unwrap_or_else(|| {
                let mut max_progress: u8 = 0;
                let mut idx: usize = 0;
                for i in 0..NUM_CHANNELS {
                    if self.channels[i].note_index >= max_progress {
                        max_progress = self.channels[i].note_index;
                        idx = i;
                    }
                }
                idx
            })
        };

        self.channels[ch] = Channel {
            sfx_id: sfx_id as i8,
            note_index: if (0..32).contains(&offset) {
                offset as u8
            } else {
                0
            },
            sub_tick: 0.0,
            phase: 0.0,
            finished: false,
            ..Channel::new()
        };
        self.read_note(memory, ch);
    }

    pub fn play_music(&mut self, memory: &Memory, pattern: i32, fade_ms: i32, mask: i32) {
        if pattern < 0 {
            if fade_ms > 0 && self.music_state.playing {
                self.music_state.fade_samples = ((fade_ms as u64) * SAMPLE_RATE as u64 / 1000) as u32;
                self.music_state.fade_progress = 0;
                self.music_state.fade_out = true;
            } else {
                self.stop_music_channels();
                self.music_state.pattern = -1;
                self.music_state.playing = false;
            }
            return;
        }
        if pattern >= 64 {
            return;
        }
        self.music_state = MusicState {
            pattern: pattern as i16,
            tick: 0,
            channel_mask: if mask > 0 { mask as u8 } else { 0xF },
            playing: true,
            total_patterns: 0,
            ..MusicState::new()
        };
        self.start_music_pattern(memory);
    }

    fn start_music_pattern(&mut self, memory: &Memory) {
        let p = self.music_state.pattern;
        if !(0..64).contains(&p) {
            return;
        }
        let base = memory::ADDR_MUSIC as usize + p as usize * 4;
        if memory.ram[base] & 0x80 != 0 {
            self.music_state.loop_back = p;
        }
        for ch_i in 0..4 {
            if self.music_state.channel_mask & (1 << ch_i) == 0 {
                continue;
            }
            let byte = memory.ram[base + ch_i];
            if byte & 0x40 != 0 {
                continue;
            }
            self.channels[ch_i] = Channel {
                sfx_id: (byte & 0x3F) as i8,
                note_index: 0,
                sub_tick: 0.0,
                phase: 0.0,
                finished: false,
                ..Channel::new()
            };
            self.read_note(memory, ch_i);
        }
    }

    fn stop_channel(&mut self, ch: usize) {
        self.channels[ch].sfx_id = -1;
        self.channels[ch].finished = true;
    }

    fn stop_music_channels(&mut self) {
        for i in 0..NUM_CHANNELS {
            if self.music_state.channel_mask & (1 << i) != 0 {
                self.stop_channel(i);
            }
        }
    }

    fn release_loop(&mut self, ch: usize) {
        if self.channels[ch].sfx_id < 0 {
            return;
        }
        self.channels[ch].loop_released = true;
    }

    fn read_note(&mut self, memory: &Memory, ch: usize) {
        let sfx_id = self.channels[ch].sfx_id;
        if sfx_id < 0 {
            return;
        }
        let base = memory::ADDR_SFX as usize + sfx_id as usize * 68;
        let note_addr = base + 4 + self.channels[ch].note_index as usize * 2;
        let lo = memory.ram[note_addr];
        let hi = memory.ram[note_addr + 1];
        let val16 = (lo as u16) | ((hi as u16) << 8);

        let pitch = (val16 & 0x3F) as u8;
        let waveform = ((val16 >> 6) & 0x7) as u8;
        let volume = ((val16 >> 9) & 0x7) as u8;
        let effect = ((val16 >> 12) & 0x7) as u8;
        let custom = (val16 >> 15) & 1 != 0;

        let freq = note_to_freq(pitch);
        let vol = volume as f32 / 7.0;
        let c = &mut self.channels[ch];
        c.prev_frequency = c.base_frequency;
        c.base_volume = vol;
        c.volume = vol;
        c.effect = effect;
        c.base_frequency = freq;
        c.frequency = freq;
        c.note_progress = 0.0;
        c.custom = custom;

        if custom {
            c.inst_sfx_id = waveform;
            let should_retrigger = pitch != c.prev_pitch || c.prev_vol == 0 || effect == 3;
            if should_retrigger {
                c.inst_note_index = 0;
                c.inst_sub_tick = 0.0;
                c.inst_phase = 0.0;
            }
            c.waveform = Waveform::Triangle;
        } else {
            c.waveform = Waveform::from(waveform);
        }
        c.prev_pitch = pitch;
        c.prev_vol = volume;
    }

    fn read_note_raw(&self, memory: &Memory, sfx_id: u8, note_index: u8) -> (u8, u8, u8, u8) {
        let base = memory::ADDR_SFX as usize + sfx_id as usize * 68;
        let note_addr = base + 4 + note_index as usize * 2;
        let lo = memory.ram[note_addr];
        let hi = memory.ram[note_addr + 1];
        let val16 = (lo as u16) | ((hi as u16) << 8);
        (
            (val16 & 0x3F) as u8,
            ((val16 >> 6) & 0x7) as u8,
            ((val16 >> 9) & 0x7) as u8,
            ((val16 >> 12) & 0x7) as u8,
        )
    }

    pub fn generate_sample(&mut self, memory: &Memory) -> f32 {
        let mut music_mix: f32 = 0.0;
        let mut sfx_mix: f32 = 0.0;

        for i in 0..NUM_CHANNELS {
            if self.channels[i].finished || self.channels[i].sfx_id < 0 {
                continue;
            }
            let is_music_ch = self.music_state.playing
                && (self.music_state.channel_mask & (1 << i) != 0);
            let sfx_base = memory::ADDR_SFX as usize + self.channels[i].sfx_id as usize * 68;

            // Compute effect-adjusted freq/vol
            let mut freq = self.channels[i].base_frequency;
            let mut vol = self.channels[i].base_volume;
            let t = self.channels[i].note_progress;

            match self.channels[i].effect {
                0 => {}
                1 => {
                    freq = self.channels[i].prev_frequency
                        + (self.channels[i].base_frequency - self.channels[i].prev_frequency)
                            * t as f64;
                }
                2 => {
                    let lfo = (t as f64 * core::f64::consts::TAU * 7.5).sin();
                    freq = self.channels[i].base_frequency * (1.0 + lfo * 0.025);
                }
                3 => {
                    freq = self.channels[i].base_frequency * (1.0 - t as f64);
                }
                4 => {
                    vol = self.channels[i].base_volume * t;
                }
                5 => {
                    vol = self.channels[i].base_volume * (1.0 - t);
                }
                6 => {
                    let note_group = self.channels[i].note_index & 0xFC;
                    let lfo_phase = ((t as f64 * 8.0) % 1.0) as f32;
                    let lfo_step = ((lfo_phase * 4.0) as u8).min(3);
                    let target = note_group + lfo_step;
                    if target < 32 {
                        let addr = sfx_base + 4 + target as usize * 2;
                        let lo2 = memory.ram[addr];
                        let hi2 = memory.ram[addr + 1];
                        let pitch2 = ((lo2 as u32 | ((hi2 as u32) << 8)) & 0x3F) as u8;
                        freq = note_to_freq(pitch2);
                    }
                }
                7 => {
                    let note_group = self.channels[i].note_index & 0xFC;
                    let lfo_phase = ((t as f64 * 4.0) % 1.0) as f32;
                    let lfo_step = ((lfo_phase * 4.0) as u8).min(3);
                    let target = note_group + lfo_step;
                    if target < 32 {
                        let addr = sfx_base + 4 + target as usize * 2;
                        let lo2 = memory.ram[addr];
                        let hi2 = memory.ram[addr + 1];
                        let pitch2 = ((lo2 as u32 | ((hi2 as u32) << 8)) & 0x3F) as u8;
                        freq = note_to_freq(pitch2);
                    }
                }
                _ => {}
            }

            self.channels[i].frequency = freq;
            self.channels[i].volume = vol;

            let sample: f32;
            if self.channels[i].custom {
                let inst_base = memory::ADDR_SFX as usize
                    + self.channels[i].inst_sfx_id as usize * 68;
                let (cn_pitch, cn_wf, cn_vol, _) = self.read_note_raw(
                    memory,
                    self.channels[i].inst_sfx_id,
                    self.channels[i].inst_note_index,
                );
                let child_wf = Waveform::from(cn_wf);
                let child_vol = cn_vol as f32 / 7.0;
                let freq_shift = freq / note_to_freq(24);
                let child_freq = note_to_freq(cn_pitch) * freq_shift;
                let combined_vol = vol * child_vol;

                sample = if child_wf == Waveform::Noise {
                    self.noise_sample(child_freq as f32, i)
                } else {
                    oscillate(child_wf, self.channels[i].inst_phase)
                };
                let inst_out = sample * combined_vol * 0.5;
                if is_music_ch {
                    music_mix += inst_out;
                } else {
                    sfx_mix += inst_out;
                }
                self.channels[i].inst_phase += child_freq / SAMPLE_RATE as f64;

                let inst_speed = memory.ram[inst_base + 1];
                let inst_samples = (inst_speed as u32).max(1) * SAMPLES_PER_TICK;
                self.channels[i].inst_sub_tick += 1.0;
                if self.channels[i].inst_sub_tick >= inst_samples as f32 {
                    self.channels[i].inst_sub_tick = 0.0;
                    self.channels[i].inst_note_index += 1;
                    let inst_loop_end = memory.ram[inst_base + 3];
                    let inst_loop_start = memory.ram[inst_base + 2];
                    if self.channels[i].inst_note_index >= 32 {
                        if inst_loop_end > 0 && inst_loop_start < inst_loop_end {
                            self.channels[i].inst_note_index = inst_loop_start;
                        } else {
                            self.channels[i].inst_note_index = 31;
                        }
                    } else if inst_loop_end > 0
                        && self.channels[i].inst_note_index >= inst_loop_end
                    {
                        self.channels[i].inst_note_index = inst_loop_start;
                    }
                }
            } else {
                sample = if self.channels[i].waveform == Waveform::Noise {
                    self.noise_sample(freq as f32, i)
                } else {
                    oscillate(self.channels[i].waveform, self.channels[i].phase)
                };
                // Per-channel scaling: 0.5 keeps 4 channels under +/-2 (clipped
                // to +/-1 at the final mix). PICO-8/picolove output noticeably
                // louder than 0.25 did; 0.5 is closer.
                let ch_out = sample * self.channels[i].volume * 0.5;
                if is_music_ch {
                    music_mix += ch_out;
                } else {
                    sfx_mix += ch_out;
                }
            }

            self.channels[i].phase += self.channels[i].frequency / SAMPLE_RATE as f64;

            let speed = memory.ram[sfx_base + 1];
            let samples_per_note = (speed as u32).max(1) * SAMPLES_PER_TICK;

            self.channels[i].sub_tick += 1.0;
            self.channels[i].note_progress = self.channels[i].sub_tick / samples_per_note as f32;

            if self.channels[i].sub_tick >= samples_per_note as f32 {
                self.channels[i].sub_tick = 0.0;
                self.channels[i].note_index += 1;

                if self.music_state.playing
                    && self.music_state.channel_mask & (1 << i) != 0
                {
                    let mut first = 0usize;
                    for c_i in 0..NUM_CHANNELS {
                        if self.music_state.channel_mask & (1 << c_i) != 0
                            && !self.channels[c_i].finished
                        {
                            first = c_i;
                            break;
                        }
                    }
                    if i == first {
                        self.music_state.tick += 1;
                    }
                }

                let loop_end = memory.ram[sfx_base + 3];
                let loop_start = memory.ram[sfx_base + 2];

                if self.channels[i].note_index >= 32 {
                    self.channels[i].finished = true;
                    self.advance_music(memory);
                } else if !self.channels[i].loop_released
                    && loop_end > 0
                    && self.channels[i].note_index >= loop_end
                {
                    if loop_start < loop_end {
                        self.channels[i].note_index = loop_start;
                    } else {
                        self.channels[i].finished = true;
                        self.advance_music(memory);
                    }
                }

                if !self.channels[i].finished {
                    self.read_note(memory, i);
                }
            }
        }

        if self.music_state.fade_out && self.music_state.fade_samples > 0 {
            self.music_state.fade_progress += 1;
            if self.music_state.fade_progress >= self.music_state.fade_samples {
                self.stop_music_channels();
                self.music_state.pattern = -1;
                self.music_state.playing = false;
                self.music_state.fade_out = false;
                self.music_state.fade_samples = 0;
                music_mix = 0.0;
            }
            if self.music_state.fade_samples > 0 {
                let fade_vol = 1.0
                    - self.music_state.fade_progress as f32
                        / self.music_state.fade_samples as f32;
                music_mix *= fade_vol;
            }
        }

        (music_mix + sfx_mix).clamp(-1.0, 1.0)
    }

    fn noise_sample(&mut self, freq: f32, ch: usize) -> f32 {
        let scale = freq / MAX_FREQ;
        let prev_s = self.channels[ch].noise_sample;
        self.noise_seed ^= self.noise_seed << 13;
        self.noise_seed ^= self.noise_seed >> 17;
        self.noise_seed ^= self.noise_seed << 5;
        let signed = self.noise_seed as i32;
        let rand = signed as f32 / 2147483648.0;
        self.channels[ch].noise_prev_sample = self.channels[ch].noise_sample;
        self.channels[ch].noise_sample =
            (self.channels[ch].noise_prev_sample + scale * rand) / (1.0 + scale);
        ((prev_s + self.channels[ch].noise_sample) * 4.0 / 3.0 * (1.75 - scale))
            .clamp(-1.0, 1.0)
            * 0.7
    }

    fn advance_music(&mut self, memory: &Memory) {
        if self.music_state.pattern < 0 {
            return;
        }
        for ch_i in 0..4 {
            if self.music_state.channel_mask & (1 << ch_i) == 0 {
                continue;
            }
            // Disabled channels (0x40 bit set in pattern byte) were never
            // started, so their `finished` flag stays false forever; treat
            // them as already-done by skipping them.
            if self.channels[ch_i].sfx_id < 0 {
                continue;
            }
            if !self.channels[ch_i].finished {
                return;
            }
        }
        let base = memory::ADDR_MUSIC as usize + self.music_state.pattern as usize * 4;
        let loop_back = memory.ram[base + 1] & 0x80 != 0;
        let stop = memory.ram[base + 2] & 0x80 != 0;

        if stop {
            self.music_state.pattern = -1;
            self.music_state.playing = false;
            return;
        }
        if loop_back && self.music_state.loop_back >= 0 {
            self.music_state.pattern = self.music_state.loop_back;
        } else {
            self.music_state.pattern += 1;
            if self.music_state.pattern >= 64 {
                self.music_state.pattern = -1;
                self.music_state.playing = false;
                return;
            }
        }
        self.music_state.tick = 0;
        self.music_state.total_patterns += 1;
        self.start_music_pattern(memory);
    }
}

fn note_to_freq(note: u8) -> f64 {
    let n = (note & 0x3F) as f64;
    65.41 * 2f64.powf(n / 12.0)
}

fn oscillate(waveform: Waveform, phase: f64) -> f32 {
    match waveform {
        Waveform::Triangle => {
            let p = (phase % 1.0).abs() as f32;
            (if p < 0.5 { p * 4.0 - 1.0 } else { 3.0 - p * 4.0 }) * 0.7
        }
        Waveform::TiltedSaw => {
            let t = (phase % 1.0).abs() as f32;
            (if t < 0.875 {
                t * 16.0 / 7.0 - 1.0
            } else {
                (1.0 - t) * 16.0 - 1.0
            }) * 0.7
        }
        Waveform::Saw => {
            let p = (phase % 1.0).abs() as f32;
            (p - 0.5) * 0.9
        }
        Waveform::Square => {
            let p = (phase % 1.0).abs() as f32;
            if p < 0.5 { 1.0 / 3.0 } else { -1.0 / 3.0 }
        }
        Waveform::Pulse => {
            let p = (phase % 1.0).abs() as f32;
            if p < 0.3125 { 1.0 / 3.0 } else { -1.0 / 3.0 }
        }
        Waveform::Organ => {
            let x = phase * 4.0;
            let t1 = ((x % 2.0) - 1.0).abs() as f32;
            let t2 = ((x * 0.5 % 2.0) - 1.0).abs() as f32;
            (t1 - 0.5 + (t2 - 0.5) / 2.0 - 0.1) * 0.7
        }
        Waveform::Noise => 0.0,
        Waveform::Phaser => {
            let x = phase * 2.0;
            let t1 = ((x % 2.0) - 1.0).abs() as f32;
            let t2 = ((x * 127.0 / 128.0 % 2.0) - 1.0).abs() as f32;
            t1 - 0.5 + (t2 - 0.5) / 2.0 - 0.25
        }
    }
}
