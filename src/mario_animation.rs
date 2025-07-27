use ratatui::{
    layout::Rect,
    style::Color,
    symbols::Marker,
    widgets::canvas::{Canvas, Circle, Context, Line, Rectangle},
};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use rodio::{OutputStream, Sink, Source};
use std::f32::consts::PI;

pub struct MarioAnimation {
    mario_x: f64,
    mario_y: f64,
    mario_vx: f64,
    mario_vy: f64,
    
    mushroom_x: f64,
    mushroom_y: f64,
    mushroom_vy: f64, // For mushroom falling
    mushroom_hit: bool,
    mushroom_exploding: bool,
    mushroom_particles: Vec<Particle>,
    
    bricks: Vec<Brick>,
    bricks_hit: bool,
    animation_frame: u32,
    started: bool,
    start_time: Option<Instant>,
    ground_y: f64,
    
    // Audio system
    _stream: Option<OutputStream>,
    music_sink: Option<Arc<Mutex<Sink>>>,
    sfx_sink: Option<Arc<Mutex<Sink>>>,
    music_started: bool,
}

#[derive(Clone)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    color: Color,
}

#[derive(Clone)]
struct Brick {
    x: f64,
    y: f64,
    visible: bool,
    breaking: bool,
    break_particles: Vec<Particle>,
}

impl MarioAnimation {
    pub fn new() -> Self {
        let ground_y = 10.0;
        let mushroom_x = 120.0;
        let mushroom_y = 75.0; // High up in the brick block
        
        // Create bricks around mushroom - these contain the mushroom
        let mut bricks = Vec::new();
        for i in -2..=2 {
            bricks.push(Brick {
                x: mushroom_x + (i as f64 * 8.0),
                y: mushroom_y - 2.0, // Bricks are slightly below mushroom
                visible: true,
                breaking: false,
                break_particles: Vec::new(),
            });
        }
        
        // Initialize audio system for music and sound effects
        let (stream, music_sink, sfx_sink) = match OutputStream::try_default() {
            Ok((stream, stream_handle)) => {
                match (Sink::try_new(&stream_handle), Sink::try_new(&stream_handle)) {
                    (Ok(music_sink), Ok(sfx_sink)) => (
                        Some(stream),
                        Some(Arc::new(Mutex::new(music_sink))),
                        Some(Arc::new(Mutex::new(sfx_sink)))
                    ),
                    _ => (None, None, None)
                }
            }
            Err(_) => (None, None, None)
        };
        
        Self {
            mario_x: 20.0,
            mario_y: ground_y,
            mario_vx: 2.0,
            mario_vy: 0.0,
            
            mushroom_x,
            mushroom_y,
            mushroom_vy: 0.0,
            mushroom_hit: false,
            mushroom_exploding: false,
            mushroom_particles: Vec::new(),
            
            bricks,
            bricks_hit: false,
            animation_frame: 0,
            started: false,
            start_time: None,
            ground_y,
            
            _stream: stream,
            music_sink,
            sfx_sink,
            music_started: false,
        }
    }
    
    pub fn start(&mut self) {
        self.started = true;
        self.start_time = Some(Instant::now());
        self.start_mario_theme();
    }
    
    pub fn is_finished(&self) -> bool {
        if let Some(start_time) = self.start_time {
            start_time.elapsed() > Duration::from_secs(10) // Longer duration for full sequence
        } else {
            false
        }
    }
    
    pub fn update(&mut self) {
        if !self.started {
            return;
        }
        
        self.animation_frame += 1;
        
        // Mario physics
        self.mario_x += self.mario_vx;
        self.mario_y += self.mario_vy;
        
        // Gravity for Mario
        if self.mario_y > self.ground_y {
            self.mario_vy -= 1.5; // Gravity
        } else {
            self.mario_y = self.ground_y;
            if self.mario_vy < 0.0 {
                self.mario_vy = 0.0;
            }
        }
        
        // Jump when approaching the brick area (Mario needs to reach the bricks)
        if !self.bricks_hit && self.mario_x > self.mushroom_x - 30.0 && self.mario_x < self.mushroom_x - 5.0 && self.mario_y <= self.ground_y + 1.0 {
            self.mario_vy = 15.0; // High jump to reach the bricks above
            self.play_jump_sound();
        }
        
        // Check collision with bricks (Mario hits bricks from below)
        if !self.bricks_hit && self.mario_vy > 0.0 { // Mario is jumping up
            for brick in &self.bricks {
                if brick.visible && 
                   self.mario_x > brick.x - 4.0 && 
                   self.mario_x < brick.x + 4.0 &&
                   self.mario_y >= brick.y - 8.0 && // Mario reaches the brick level
                   self.mario_y <= brick.y - 3.0 {
                    self.hit_bricks();
                    break;
                }
            }
        }
        
        // Mushroom physics after bricks are hit
        if self.bricks_hit && !self.mushroom_hit {
            if self.mushroom_y > self.ground_y + 5.0 {
                self.mushroom_vy += 0.5; // Gravity acceleration
                self.mushroom_y -= self.mushroom_vy; // Fall down
            } else {
                // Mushroom reaches ground
                self.mushroom_y = self.ground_y + 5.0;
                if !self.mushroom_exploding {
                    self.explode_mushroom();
                }
            }
        }
        
        // Update particles
        self.update_particles();
        
        // Update brick particles
        for brick in &mut self.bricks {
            for particle in &mut brick.break_particles {
                particle.x += particle.vx;
                particle.y += particle.vy;
                particle.vy -= 0.3; // Gravity on particles
                particle.life -= 0.02;
            }
            brick.break_particles.retain(|p| p.life > 0.0);
        }
        
        // Continue moving Mario after hitting bricks
        if self.bricks_hit && self.mario_x < 200.0 {
            self.mario_vx = 1.5;
        }
    }
    
    fn hit_bricks(&mut self) {
        self.bricks_hit = true;
        
        // Play brick break sound
        self.play_brick_break_sound();
        
        // Break all bricks with explosion effect
        for brick in &mut self.bricks {
            brick.visible = false;
            brick.breaking = true;
            
            // Create brick particles
            for j in 0..12 {
                let angle = (j as f64) * 0.524; // 2π/12
                let speed = 2.0 + (j as f64 % 3.0);
                brick.break_particles.push(Particle {
                    x: brick.x,
                    y: brick.y,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed + 3.0,
                    life: 1.0,
                    color: Color::Rgb(139, 69, 19), // Brown brick color
                });
            }
        }
        
        // Mario gets a little bounce back from hitting the bricks
        self.mario_vy = -2.0;
    }
    
    fn explode_mushroom(&mut self) {
        self.mushroom_hit = true;
        self.mushroom_exploding = true;
        
        // Play power-up sound
        self.play_powerup_sound();
        
        // Create mushroom explosion particles
        for i in 0..25 {
            let angle = (i as f64) * 0.251; // 2π/25
            let speed = 2.0 + (i as f64 % 4.0);
            self.mushroom_particles.push(Particle {
                x: self.mushroom_x,
                y: self.mushroom_y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed + 2.0,
                life: 1.0,
                color: if i % 3 == 0 { 
                    Color::Red 
                } else if i % 3 == 1 { 
                    Color::Yellow 
                } else { 
                    Color::White 
                },
            });
        }
    }
    
    fn update_particles(&mut self) {
        for particle in &mut self.mushroom_particles {
            particle.x += particle.vx;
            particle.y += particle.vy;
            particle.vy -= 0.2; // Gravity
            particle.life -= 0.015;
        }
        self.mushroom_particles.retain(|p| p.life > 0.0);
    }
    
    pub fn render(&self, _area: Rect) -> Canvas<impl Fn(&mut Context)> {
        Canvas::default()
            .marker(Marker::Braille)
            .x_bounds([0.0, 240.0])
            .y_bounds([0.0, 100.0])
            .paint(|ctx| {
                // Draw ground
                ctx.draw(&Line {
                    x1: 0.0,
                    y1: self.ground_y - 2.0,
                    x2: 240.0,
                    y2: self.ground_y - 2.0,
                    color: Color::Green,
                });
                
                // Draw background pipes
                self.draw_pipes(ctx);
                
                // Draw bricks (only if not broken)
                for brick in &self.bricks {
                    if brick.visible && !brick.breaking {
                        self.draw_brick(ctx, brick.x, brick.y);
                    }
                    
                    // Draw brick particles
                    for particle in &brick.break_particles {
                        ctx.draw(&Circle {
                            x: particle.x,
                            y: particle.y,
                            radius: 1.0,
                            color: particle.color,
                        });
                    }
                }
                
                // Draw mushroom (visible until it explodes)
                if !self.mushroom_exploding {
                    self.draw_mushroom(ctx, self.mushroom_x, self.mushroom_y);
                }
                
                // Draw mushroom particles
                for particle in &self.mushroom_particles {
                    ctx.draw(&Circle {
                        x: particle.x,
                        y: particle.y,
                        radius: 1.5,
                        color: particle.color,
                    });
                }
                
                // Draw Mario
                self.draw_mario(ctx, self.mario_x, self.mario_y);
                
                // Draw visual effects
                // if self.bricks_hit && !self.mushroom_exploding {
                //     // Show "BREAK!" text when bricks are hit
                //     ctx.print(self.mushroom_x - 15.0, self.mushroom_y + 10.0, "BREAK!");
                // }
                
                // if self.mushroom_exploding {
                //     // Show score and power-up text
                //     ctx.print(self.mushroom_x - 10.0, self.mushroom_y + 15.0, "100");
                //     ctx.print(self.mario_x - 15.0, self.mario_y + 10.0, "SUPER!");
                // }
                
                // Flash effect when Mario hits bricks
                if self.bricks_hit && !self.mushroom_hit && self.animation_frame % 8 < 4 {
                    for brick in &self.bricks {
                        if !brick.visible {
                            ctx.draw(&Circle {
                                x: brick.x,
                                y: brick.y,
                                radius: 4.0,
                                color: Color::Yellow,
                            });
                        }
                    }
                }
                
                // Draw title
                // ctx.print(10.0, 90.0, "🍅 CYBER TOMATO - Mario Brick Breaking Animation 🍅");
            })
    }
    
    fn draw_mario(&self, ctx: &mut Context, x: f64, y: f64) {
        // Mario body (simplified)
        ctx.draw(&Circle {
            x,
            y: y + 3.0,
            radius: 3.0,
            color: Color::Red,
        });
        
        // Mario head
        ctx.draw(&Circle {
            x,
            y: y + 7.0,
            radius: 2.5,
            color: Color::Rgb(255, 220, 177), // Skin color
        });
        
        // Mario hat
        ctx.draw(&Circle {
            x,
            y: y + 8.5,
            radius: 2.0,
            color: Color::Red,
        });
        
        // Mario legs (simple lines) - walking animation
        if self.animation_frame % 10 < 5 {
            // Walking animation - leg positions
            ctx.draw(&Line {
                x1: x - 1.0,
                y1: y,
                x2: x - 2.0,
                y2: y - 3.0,
                color: Color::Blue,
            });
            ctx.draw(&Line {
                x1: x + 1.0,
                y1: y,
                x2: x + 2.0,
                y2: y - 2.0,
                color: Color::Blue,
            });
        } else {
            ctx.draw(&Line {
                x1: x - 1.0,
                y1: y,
                x2: x - 1.5,
                y2: y - 2.0,
                color: Color::Blue,
            });
            ctx.draw(&Line {
                x1: x + 1.0,
                y1: y,
                x2: x + 1.5,
                y2: y - 3.0,
                color: Color::Blue,
            });
        }
    }
    
    fn draw_mushroom(&self, ctx: &mut Context, x: f64, y: f64) {
        // Mushroom cap
        ctx.draw(&Circle {
            x,
            y: y + 2.0,
            radius: 4.0,
            color: Color::Red,
        });
        
        // Mushroom spots
        ctx.draw(&Circle {
            x: x - 2.0,
            y: y + 3.0,
            radius: 0.8,
            color: Color::White,
        });
        ctx.draw(&Circle {
            x: x + 1.5,
            y: y + 2.5,
            radius: 0.6,
            color: Color::White,
        });
        
        // Mushroom stem
        ctx.draw(&Rectangle {
            x: x - 1.0,
            y: y - 2.0,
            width: 2.0,
            height: 4.0,
            color: Color::Rgb(255, 248, 220), // Beige
        });
    }
    
    fn draw_brick(&self, ctx: &mut Context, x: f64, y: f64) {
        ctx.draw(&Rectangle {
            x: x - 3.0,
            y: y - 1.5,
            width: 6.0,
            height: 3.0,
            color: Color::Rgb(139, 69, 19), // Brown
        });
        
        // Brick lines for texture
        ctx.draw(&Line {
            x1: x - 3.0,
            y1: y,
            x2: x + 3.0,
            y2: y,
            color: Color::Rgb(160, 82, 45),
        });
        ctx.draw(&Line {
            x1: x,
            y1: y - 1.5,
            x2: x,
            y2: y + 1.5,
            color: Color::Rgb(160, 82, 45),
        });
    }
    
    fn draw_pipes(&self, ctx: &mut Context) {
        // Background pipes for Mario theme
        let pipe_positions = [200.0, 220.0];
        
        for &pipe_x in &pipe_positions {
            // Pipe body
            ctx.draw(&Rectangle {
                x: pipe_x - 4.0,
                y: self.ground_y - 2.0,
                width: 8.0,
                height: 20.0,
                color: Color::Green,
            });
            
            // Pipe top
            ctx.draw(&Rectangle {
                x: pipe_x - 5.0,
                y: self.ground_y + 16.0,
                width: 10.0,
                height: 3.0,
                color: Color::Green,
            });
        }
    }
    
    fn start_mario_theme(&mut self) {
        if self.music_started {
            return;
        }
        self.music_started = true;
        
        if let Some(ref sink) = self.music_sink {
            let sink = sink.lock().unwrap();
            
            // Mario Bros main theme melody (simplified)
            let mario_theme = vec![
                (659.25, 150), // E5
                (659.25, 150), // E5
                (0.0, 150),    // Rest
                (659.25, 150), // E5
                (0.0, 150),    // Rest
                (523.25, 150), // C5
                (659.25, 150), // E5
                (0.0, 150),    // Rest
                (783.99, 150), // G5
                (0.0, 450),    // Rest
                (392.00, 150), // G4
                (0.0, 450),    // Rest
                
                (523.25, 150), // C5
                (0.0, 300),    // Rest
                (392.00, 150), // G4
                (0.0, 300),    // Rest
                (329.63, 150), // E4
                (0.0, 300),    // Rest
                (440.00, 150), // A4
                (0.0, 150),    // Rest
                (493.88, 150), // B4
                (0.0, 150),    // Rest
                (466.16, 150), // A#4
                (440.00, 150), // A4
                (0.0, 150),    // Rest
                
                (392.00, 200), // G4
                (659.25, 200), // E5
                (783.99, 200), // G5
                (880.00, 150), // A5
                (0.0, 150),    // Rest
                (698.46, 150), // F5
                (783.99, 150), // G5
                (0.0, 150),    // Rest
                (659.25, 150), // E5
                (0.0, 150),    // Rest
                (523.25, 150), // C5
                (587.33, 150), // D5
                (493.88, 150), // B4
                (0.0, 300),    // Rest
            ];
            
            for (freq, duration_ms) in mario_theme {
                if freq > 0.0 {
                    let source = MarioTone::new(freq, Duration::from_millis(duration_ms));
                    sink.append(source);
                } else {
                    // Rest/silence
                    let silence = rodio::source::Zero::<f32>::new(1, 44100)
                        .take_duration(Duration::from_millis(duration_ms))
                        .buffered();
                    sink.append(silence);
                }
            }
        }
    }
    
    fn play_jump_sound(&self) {
        if let Some(ref sink) = self.sfx_sink {
            let sink = sink.lock().unwrap();
            let jump_tones = [
                (523.25, Duration::from_millis(100)), // C5
                (659.25, Duration::from_millis(100)), // E5
            ];
            self.play_sound_effect(&sink, &jump_tones);
        }
    }
    
    fn play_brick_break_sound(&self) {
        if let Some(ref sink) = self.sfx_sink {
            let sink = sink.lock().unwrap();
            let break_tones = [
                (1046.50, Duration::from_millis(80)), // C6
                (0.0, Duration::from_millis(20)),     // Rest
                (1174.66, Duration::from_millis(80)), // D6
                (0.0, Duration::from_millis(20)),     // Rest
                (1318.51, Duration::from_millis(120)), // E6
            ];
            self.play_sound_effect(&sink, &break_tones);
        }
    }
    
    fn play_powerup_sound(&self) {
        if let Some(ref sink) = self.sfx_sink {
            let sink = sink.lock().unwrap();
            let powerup_tones = [
                (392.00, Duration::from_millis(100)), // G4
                (523.25, Duration::from_millis(100)), // C5
                (659.25, Duration::from_millis(100)), // E5
                (783.99, Duration::from_millis(100)), // G5
                (1046.50, Duration::from_millis(100)), // C6
                (1318.51, Duration::from_millis(300)), // E6
            ];
            self.play_sound_effect(&sink, &powerup_tones);
        }
    }
    
    fn play_sound_effect(&self, sink: &std::sync::MutexGuard<Sink>, tones: &[(f32, Duration)]) {
        for (freq, dur) in tones {
            if *freq == 0.0 {
                let silence = rodio::source::Zero::<f32>::new(1, 44100)
                    .take_duration(*dur)
                    .buffered();
                sink.append(silence);
            } else {
                let source = MarioTone::new(*freq, *dur);
                sink.append(source);
            }
        }
    }
}

// Custom audio source for Mario-style tones
struct MarioTone {
    freq: f32,
    duration: Duration,
    sample_rate: u32,
    sample_idx: usize,
    total_samples: usize,
}

impl MarioTone {
    fn new(freq: f32, duration: Duration) -> Self {
        let sample_rate = 44100;
        let total_samples = (duration.as_secs_f32() * sample_rate as f32) as usize;
        Self {
            freq,
            duration,
            sample_rate,
            sample_idx: 0,
            total_samples,
        }
    }
}

impl Iterator for MarioTone {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sample_idx >= self.total_samples {
            return None;
        }

        let t = self.sample_idx as f32 / self.sample_rate as f32;
        let phase = 2.0 * PI * self.freq * t;
        
        // Square wave with envelope for classic Mario sound
        let square_wave = if (phase % (2.0 * PI)) < PI { 0.3 } else { -0.3 };
        
        // Envelope with decay
        let envelope = (-3.0 * t).exp();
        
        let sample = square_wave * envelope;
        self.sample_idx += 1;
        Some(sample)
    }
}

impl Source for MarioTone {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.total_samples - self.sample_idx)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.duration)
    }
}