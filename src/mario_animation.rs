use ratatui::{
    layout::Rect,
    style::Color,
    symbols::Marker,
    widgets::canvas::{Canvas, Circle, Context, Line, Rectangle},
};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct MarioAnimation {
    mario_x: f64,
    mario_y: f64,
    mario_vx: f64,
    mario_vy: f64,
    
    mushroom_x: f64,
    mushroom_y: f64,
    mushroom_hit: bool,
    mushroom_exploding: bool,
    mushroom_particles: Vec<Particle>,
    
    bricks: Vec<Brick>,
    animation_frame: u32,
    started: bool,
    start_time: Option<Instant>,
    ground_y: f64,
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
        let mushroom_y = 60.0;
        
        // Create bricks around mushroom
        let mut bricks = Vec::new();
        for i in -2..=2 {
            bricks.push(Brick {
                x: mushroom_x + (i as f64 * 8.0),
                y: mushroom_y + 10.0,
                visible: true,
                breaking: false,
                break_particles: Vec::new(),
            });
        }
        
        Self {
            mario_x: 20.0,
            mario_y: ground_y,
            mario_vx: 2.0,
            mario_vy: 0.0,
            
            mushroom_x,
            mushroom_y,
            mushroom_hit: false,
            mushroom_exploding: false,
            mushroom_particles: Vec::new(),
            
            bricks,
            animation_frame: 0,
            started: false,
            start_time: None,
            ground_y,
        }
    }
    
    pub fn start(&mut self) {
        self.started = true;
        self.start_time = Some(Instant::now());
    }
    
    pub fn is_finished(&self) -> bool {
        if let Some(start_time) = self.start_time {
            start_time.elapsed() > Duration::from_secs(8)
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
        
        // Gravity
        if self.mario_y > self.ground_y {
            self.mario_vy -= 1.5; // Gravity
        } else {
            self.mario_y = self.ground_y;
            if self.mario_vy < 0.0 {
                self.mario_vy = 0.0;
            }
        }
        
        // Jump when approaching mushroom
        if !self.mushroom_hit && self.mario_x > self.mushroom_x - 30.0 && self.mario_x < self.mushroom_x - 10.0 && self.mario_y <= self.ground_y + 1.0 {
            self.mario_vy = 8.0; // Jump velocity
        }
        
        // Check collision with mushroom
        if !self.mushroom_hit && 
           self.mario_x > self.mushroom_x - 5.0 && 
           self.mario_x < self.mushroom_x + 5.0 &&
           self.mario_y > self.mushroom_y - 5.0 &&
           self.mario_y < self.mushroom_y + 5.0 {
            self.hit_mushroom();
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
        
        // Continue moving Mario after mushroom hit
        if self.mushroom_hit && self.mario_x < 200.0 {
            self.mario_vx = 1.5;
        }
    }
    
    fn hit_mushroom(&mut self) {
        self.mushroom_hit = true;
        self.mushroom_exploding = true;
        
        // Create mushroom explosion particles
        for i in 0..20 {
            let angle = (i as f64) * 0.314; // Roughly 2π/20
            let speed = 3.0 + (i as f64 % 3.0);
            self.mushroom_particles.push(Particle {
                x: self.mushroom_x,
                y: self.mushroom_y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed + 2.0,
                life: 1.0,
                color: if i % 2 == 0 { Color::Red } else { Color::Yellow },
            });
        }
        
        // Break bricks
        for brick in &mut self.bricks {
            brick.visible = false;
            brick.breaking = true;
            
            // Create brick particles
            for j in 0..8 {
                let angle = (j as f64) * 0.785; // π/4
                brick.break_particles.push(Particle {
                    x: brick.x,
                    y: brick.y,
                    vx: angle.cos() * 2.0,
                    vy: angle.sin() * 2.0 + 1.0,
                    life: 1.0,
                    color: Color::Rgb(139, 69, 19), // Brown brick color
                });
            }
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
                
                // Draw bricks
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
                
                // Draw mushroom (if not hit)
                if !self.mushroom_hit {
                    self.draw_mushroom(ctx, self.mushroom_x, self.mushroom_y);
                }
                
                // Draw mushroom particles
                for particle in &self.mushroom_particles {
                    let _alpha = (particle.life * 255.0) as u8;
                    ctx.draw(&Circle {
                        x: particle.x,
                        y: particle.y,
                        radius: 1.5,
                        color: particle.color,
                    });
                }
                
                // Draw Mario
                self.draw_mario(ctx, self.mario_x, self.mario_y);
                
                // Draw score/effects text
                if self.mushroom_hit {
                    ctx.print(self.mushroom_x - 10.0, self.mushroom_y + 15.0, "100");
                    ctx.print(self.mario_x - 15.0, self.mario_y + 10.0, "SUPER!");
                }
                
                // Draw title
                ctx.print(10.0, 90.0, "🍅 CYBER TOMATO - Mario Animation 🍅");
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
        
        // Mario legs (simple lines)
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
        
        // Brick lines
        ctx.draw(&Line {
            x1: x - 3.0,
            y1: y,
            x2: x + 3.0,
            y2: y,
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
}