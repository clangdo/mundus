use rand::{distributions::Uniform, prelude::*};

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct Dice {
    quantity: u32,
    sides: u32,
}

impl fmt::Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}d{}", self.quantity, self.sides)
    }
}

#[derive(Debug)]
pub struct Roll {
    dice: Dice,
    pub result: u32,
}

impl fmt::Display for Roll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.dice, self.result)
    }
}

impl Dice {
    pub fn new(string_repr: &str) -> Result<Dice, Box<dyn Error>> {
        let mut iter = string_repr.split('d');
        let mut dice = Dice { quantity: 0, sides: 0 };

        if let Some(string_number) = iter.next() {
            dice.quantity = string_number.parse::<u32>()?;
        } else {
            dice.quantity = 1;
        }

        if let Some(string_number) = iter.next() {
            dice.sides = string_number.parse::<u32>()?;
        } else {
            return Err("The dice had no number that could be construed as the sides".into());
        }

        if iter.next_back().is_some() {
            return Err("The dice had too many 'd' characters.".into());
        }

        Ok(dice)
    }
    
    pub fn roll(&self, rng: &mut ThreadRng) -> Roll {
        let dist = Uniform::new_inclusive(
            self.quantity,
            self.quantity * self.sides,
        );
        
        Roll {
            dice: Dice{quantity: self.quantity, sides: self.sides},
            result: rng.sample(dist),
        }
    }
}
