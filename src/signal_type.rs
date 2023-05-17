use std::str::FromStr;

use pyo3::prelude::*;

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

/// The different signals that can be generated
#[pyclass]
#[derive(Copy, Clone, Display, EnumIter, EnumString, PartialEq)]
pub enum SignalType {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    Constant,
}

#[pymethods]
impl SignalType {
    fn to_string(&self) -> &'static str {
        match self {
            SignalType::Sine => "Sine",
            SignalType::Square => "Square",
            SignalType::Triangle => "Triangle",
            SignalType::Sawtooth => "Sawtooth",
            SignalType::Constant => "Constant",
        }
    }

    #[staticmethod]
    fn parse(string: &str) -> Self {
        SignalType::from_str(string).unwrap()
    }

    #[staticmethod]
    fn from_string(string: &str) -> Self {
        SignalType::from_str(string).unwrap()
    }

    fn __repr__(&self) -> &'static str {
        self.to_string()
    }
}

#[pyfunction]
pub fn get_signal_types() -> Vec<SignalType> {
    SignalType::iter().collect()
}

pub mod generators {
    use super::SignalType;

    use core::fmt::Debug;
    use rand::Rng;
    use std::f32::consts::PI;

    /// A macro to create structs for each SignalType with the fields: amplitude, frequency, phase (all f32)
    macro_rules! signal_type_struct {
        ($($name:ident),*) => {
            $(
                #[derive(Debug)]
                pub struct $name {
                    pub minimum: f32,
                    pub maximum: f32,
                    pub amplitude: f32,
                    pub period: f32,
                    pub phase: f32,
                    pub num_bits: u8,
                    pub is_signed: bool,
                    pub scale: f32,
                    pub offset: f32
                }
            )*
        };
    }

    macro_rules! signal_type_getters {
        ($name:ident) => {
            fn get_type(&self) -> SignalType {
                SignalType::$name
            }
            fn get_minimum(&self) -> f32 {
                self.minimum
            }
            fn get_maximum(&self) -> f32 {
                self.maximum
            }
            fn get_amplitude(&self) -> f32 {
                self.amplitude
            }
            fn get_period(&self) -> f32 {
                self.period
            }
            fn get_phase(&self) -> f32 {
                self.phase
            }
            fn get_num_bits(&self) -> u8 {
                self.num_bits
            }
            fn is_signed(&self) -> bool {
                self.is_signed
            }
            fn get_scale(&self) -> f32 {
                self.scale
            }
            fn get_offset(&self) -> f32 {
                self.offset
            }
        };
    }

    // Create structs for each SignalType
    signal_type_struct!(Sine, Square, Triangle, Sawtooth, Constant);

    pub trait Signal: Send {
        fn get_type(&self) -> SignalType;
        fn get_minimum(&self) -> f32;
        fn get_maximum(&self) -> f32;
        fn get_amplitude(&self) -> f32;
        fn get_period(&self) -> f32;
        fn get_phase(&self) -> f32;
        fn get_num_bits(&self) -> u8;
        fn is_signed(&self) -> bool;
        fn get_scale(&self) -> f32;
        fn get_offset(&self) -> f32;

        fn get_type_name(&self) -> &'static str {
            self.get_type().to_string()
        }

        /// Shrink a value to only take up a certain number of bits
        /// after the scale and offset have been applied
        ///
        /// Note: the number has to remain within the range of the signal's
        /// minimum and maximum values
        fn shrink_to_fit(&self, value: f32) -> i64 {
            // Apply the reverse of the scale and offset
            let clamped = value.max(self.get_minimum()).min(self.get_maximum());
            let scaled = clamped / self.get_scale();
            let offset = scaled - self.get_offset();
            let offset = offset.round() as i64;

            let num_bits = self.get_num_bits();
            let is_signed = self.is_signed();

            let max_value = if is_signed {
                2_i64.pow(num_bits as u32 - 1) - 1
            } else {
                2_i64.pow(num_bits as u32) - 1
            };

            let min_value = if is_signed {
                -(2_i64.pow(num_bits as u32 - 1))
            } else {
                0
            };

            // Clamp the value to the range of the number of bits
            let clamped = offset.max(min_value).min(max_value);

            // Undo the scale and offset
            let clamped = (clamped as f32 + self.get_offset()) * self.get_scale();
            clamped.round() as i64
        }

        /// Calculates the fraction to use as the noise
        fn noise(&self) -> f32 {
            static NOISE: f32 = 0.1;
            let mut rng = rand::thread_rng();
            rng.gen_range(-NOISE..NOISE)
        }

        /// Calculate the value of the signal at a given time with noise
        fn calculate(&self, time: f32) -> i64;
    }

    impl Debug for dyn Signal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Signal")
                .field("type", &self.get_type_name())
                .field("minimum", &self.get_minimum())
                .field("maximum", &self.get_maximum())
                .field("amplitude", &self.get_amplitude())
                .field("period", &self.get_period())
                .field("phase", &self.get_phase())
                .field("num_bits", &self.get_num_bits())
                .field("is_signed", &self.is_signed())
                .field("scale", &self.get_scale())
                .field("offset", &self.get_offset())
                .finish()
        }
    }

    impl Signal for Sine {
        signal_type_getters!(Sine);

        fn calculate(&self, time: f32) -> i64 {
            let a = self.get_amplitude();
            let b = 2.0 * PI / self.get_period();
            let c = self.get_phase();

            let value = a * ((b * (time + c)).sin() + self.noise());
            let value = value.clamp(self.minimum, self.maximum);
            self.shrink_to_fit(value)
        }
    }

    impl Signal for Square {
        signal_type_getters!(Square);

        fn calculate(&self, time: f32) -> i64 {
            let value = {
                if (time + self.phase) % self.period < self.period / 2.0 {
                    self.amplitude
                } else {
                    -self.amplitude
                }
            };
            let value = value + self.noise() * self.get_amplitude();
            let value = value.clamp(self.minimum, self.maximum);
            self.shrink_to_fit(value)
        }
    }

    impl Signal for Triangle {
        signal_type_getters!(Triangle);

        fn calculate(&self, time: f32) -> i64 {
            let t = (time + self.phase) % self.period;
            let value = {
                if t < 0.25 {
                    self.amplitude * t * 4.0
                } else if t < 0.75 {
                    self.amplitude * (1.0 - (t - 0.25) * 4.0)
                } else {
                    self.amplitude * (t - 0.75) * 4.0 - self.amplitude
                }
            };
            let value = value + self.noise() * self.amplitude;
            let value = value.clamp(self.minimum, self.maximum);
            self.shrink_to_fit(value)
        }
    }

    impl Signal for Sawtooth {
        signal_type_getters!(Sawtooth);

        fn calculate(&self, time: f32) -> i64 {
            let t = (time + self.phase) % self.period;
            let value = self.amplitude * (t * 2.0 - 1.0);
            let value = value + self.noise() * self.amplitude;
            let value = value.clamp(self.minimum, self.maximum);
            self.shrink_to_fit(value)
        }
    }

    impl Signal for Constant {
        signal_type_getters!(Constant);

        fn calculate(&self, _time: f32) -> i64 {
            let value = self.amplitude;
            let value = value + self.noise() * self.amplitude;
            let value = value.clamp(self.minimum, self.maximum);
            self.shrink_to_fit(value)
        }
    }
}
