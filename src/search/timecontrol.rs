pub const DEFAULT_MOVE_OVERHEAD: u64 = 25;
pub const MIN_MOVE_OVERHEAD: u64 = 0;
pub const MAX_MOVE_OVERHEAD: u64 = 20000;

pub struct TimeControl {
    pub typ: TimeControlType,
    pub stable_pv: bool,
    pub aspired_time: u64,
}
impl Default for TimeControl {
    fn default() -> Self {
        TimeControl {
            typ: TimeControlType::Infinite,
            aspired_time: 0u64,
            stable_pv: true,
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
pub enum TimeControlType {
    Incremental(u64, u64),
    MoveTime(u64),
    Infinite,
    Tournament(u64, u64, usize),
}
impl TimeControlType {
    pub fn base_time(&self) -> u64 {
        match *self {
            TimeControlType::Infinite => panic!("Don't call base time on infinite!"),
            TimeControlType::MoveTime(x) => x,
            TimeControlType::Incremental(x, y) => TimeControlType::Tournament(x, y, 25).base_time(),
            TimeControlType::Tournament(x, _, mv) => x / mv as u64,
        }
    }

    pub fn increment(&self) -> u64 {
        match *self {
            TimeControlType::Incremental(_, inc) => inc,
            TimeControlType::MoveTime(_) => 0,
            TimeControlType::Infinite => panic!("Don't call this on Infinite"),
            TimeControlType::Tournament(_, inc, _) => inc,
        }
    }

    pub fn compound_time(&self) -> u64 {
        self.base_time() + self.increment()
    }
    pub fn update(&mut self, time_spent: u64, tournament_info: Option<(usize, u64)>) {
        match self {
            TimeControlType::Incremental(left, inc) => {
                assert!(*left > time_spent);
                *self = TimeControlType::Incremental(*left - time_spent + *inc, *inc);
            }
            TimeControlType::Infinite => panic!("Should not call update on Infinite"),
            TimeControlType::Tournament(left, inc, movestogo) => {
                assert!(*left > time_spent);
                let mut new_left = *left - time_spent + *inc;
                if *movestogo == 0 {
                    new_left += tournament_info.unwrap().1;
                    *movestogo = tournament_info.unwrap().0;
                } else {
                    *movestogo -= 1;
                }
                *self = TimeControlType::Tournament(new_left, *inc, *movestogo);
            }
            _ => {}
        }
    }

    pub fn time_left(&self) -> u64 {
        match self {
            TimeControlType::Incremental(left, _) => *left,
            TimeControlType::MoveTime(left) => *left,
            TimeControlType::Infinite => panic!("Should not call time_left on Infinite"),
            TimeControlType::Tournament(left, _, _) => *left,
        }
    }

    pub fn to_go(&self, white: bool) -> String {
        match self {
            TimeControlType::Incremental(time_left, inc) => {
                if white {
                    format!("wtime {} winc {}", time_left, inc)
                } else {
                    format!("btime {} binc {}", time_left, inc)
                }
            }
            TimeControlType::MoveTime(time) => format!("movetime {}", time),
            TimeControlType::Infinite => "infinite".to_owned(),
            TimeControlType::Tournament(time_left, inc, movestogo) => {
                if white {
                    format!("wtime {} winc {} movestogo {}", time_left, inc, movestogo)
                } else {
                    format!("btime {} binc {} movestogo {}", time_left, inc, movestogo)
                }
            }
        }
    }
}

impl TimeControl {
    pub fn update_aspired_time(&mut self, mult: f64) {
        if self.typ != TimeControlType::Infinite {
            let mut new = (self.aspired_time as f64 * mult) as u64;
            new = new.min((5. * self.typ.base_time() as f64 + self.typ.increment() as f64) as u64);
            new = new.max((0.4 * self.typ.compound_time() as f64) as u64);
            new = new.min(self.typ.time_left() / 3 + self.typ.increment());
            println!(
                "Update aspire. Old: {} ; Mult: {}; New: {}",
                self.aspired_time, mult, new
            );
            self.aspired_time = new;
        }
    }
    pub fn update_type(&mut self, typ: TimeControlType) {
        self.typ = typ;
        if self.typ != TimeControlType::Infinite {
            self.aspired_time = self.typ.compound_time();
            self.update_aspired_time(1.0);
        }
        self.stable_pv = false;
    }

    pub fn time_over(&self, time_spent: u64, move_overhead: u64) -> bool {
        match self.typ {
            TimeControlType::Incremental(_, _) | TimeControlType::Tournament(_, _, _) => {
                time_spent + 4 * move_overhead > self.typ.time_left()
                    || (self.stable_pv
                        || time_spent + move_overhead
                            > (self.typ.compound_time() as f64 + 2. * self.typ.base_time() as f64)
                                as u64)
                        && time_spent + move_overhead > self.aspired_time
            }
            TimeControlType::Infinite => false,
            TimeControlType::MoveTime(x) => time_spent + move_overhead > x,
        }
    }

    pub fn as_string(&self) -> String {
        let mut res_str: String = String::new();
        if let TimeControlType::Incremental(mytime, myinc) = self.typ {
            res_str.push_str(&format!("My Time: {}\n", mytime));
            res_str.push_str(&format!("My Inc: {}\n", myinc));
            res_str.push_str(&format!("Time I aspire to spend: {}\n", self.aspired_time));
        } else if let TimeControlType::MoveTime(time) = self.typ {
            res_str.push_str(&format!("Limited movetime: {}\n", time));
        } else if let TimeControlType::Infinite = self.typ {
            res_str.push_str("Infinite Time!\n");
        } else if let TimeControlType::Tournament(mytime, myinc, movestogo) = self.typ {
            res_str.push_str(&format!("My Time: {}\n", mytime));
            res_str.push_str(&format!("My Inc: {}\n", myinc));
            res_str.push_str(&format!("Moves to go : {}\n", movestogo));
            res_str.push_str(&format!("Time I aspire to spend: {}\n", self.aspired_time));
        }
        res_str
    }
}
