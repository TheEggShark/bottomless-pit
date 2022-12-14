use raylib::{core::{RaylibThread, RaylibHandle}, prelude::RaylibDrawHandle};
use animation::{PlayerAnimation, HitBox};
use utils::Point;

#[derive(Debug)]
pub(super) struct Attack {
    base_hitbox: HitBox, // all hitboxes should originate at 0,0
    actual: HitBox,      // then we shift this when the attack starts,
    animation: PlayerAnimation,
    base_damage: f32,
    frame_data: FrameData,
    state: AttackState,
    frame_count: i16,
}

#[derive(Debug)]
enum AttackState {
    Startup,
    Active,
    Recovery,
}

#[derive(Debug)]
pub(super) struct FrameData {
    startup: i16, // animation no hitbox
    active: i16,  // anumation + hitbox
    recovery: i16,// animation no hitbox
    on_block: i16,// how soon you can act
    on_hit: i16,  // does not include gat + special cancle
}

impl FrameData {
    pub fn new(startup: i16, active: i16, recovery: i16, on_block: i16, on_hit: i16) -> Self {
        Self {
            startup,
            active,
            recovery,
            on_block,
            on_hit,
        }
    }
}

impl Attack {
    pub fn new(base_hitbox: HitBox, path: &str, base_damage: f32, animation_frames: i16, frame_data: FrameData, rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let animation = PlayerAnimation::new(path, animation_frames, rl, thread);
        Self {
            actual: base_hitbox.copy(),
            base_hitbox,
            animation,
            base_damage,
            frame_data,
            state: AttackState::Startup,
            frame_count: 0,
        }
    }

    pub fn shift_actual(&mut self, shift_x: i32, shift_y: i32) {
        // used to have a hitbox based off where the player actually is
        self.actual.shift_x(shift_x);
        self.actual.shift_y(shift_y);
    }

    pub fn reset_actual(&mut self) {
        // resets the hitbox allowing us to change it for the next attack
        self.actual = self.base_hitbox.copy();
    }

    pub fn draw(&self, pos: Point, d_handle: &mut RaylibDrawHandle) {
        self.animation.draw(d_handle, pos);
        match self.state {
            AttackState::Active => {
                self.actual.draw_hitbox(d_handle);
            },
            _ => {},
        }
    }

    // reutrns true if the attack is over I liked this more
    // compared to a is_over functions
    pub fn update(&mut self) -> bool {
        self.frame_count += 1;
        self.animation.update(1);

        // sets the state to the approraite state based off the frame data
        if self.frame_count > self.frame_data.active {
            self.state = AttackState::Recovery
        } else if self.frame_count > self.frame_data.startup {
            self.state = AttackState::Active
        }

        // checks to see if the attack is "over"
        if self.frame_count == (self.frame_data.active + self.frame_data.recovery + self.frame_data.startup) {
            self.reset_actual();
            self.frame_count = 0;
            self.animation.set_frame(0);
            return true;
        }

        false
    }
}

#[derive(Debug)]
pub(super) enum AttackType {
    Slash,
    Kick,
}

impl AttackType {
    pub fn into_uszie(&self) -> usize {
        match self {
            Self::Slash => 0,
            Self::Kick => 1,
        }
    }
}