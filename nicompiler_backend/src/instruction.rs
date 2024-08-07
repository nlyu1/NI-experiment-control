//! Provides definitions and implementations for instruction-related functionalities.
//!
//! ## Main Structures and Enumerations:
//!
//! - [`InstrType`]: An enumeration that defines the types of instructions supported, including `CONST` for constant values and `SINE` for sinusoidal waves.
//!
//! - [`Instruction`]: Represents a general instruction composed of a type (`InstrType`) and a set of arguments (`InstrArgs`). It offers methods for creating specific instruction types conveniently and for evaluating them.
//!
//! - [`InstrBook`]: Manages an instruction along with its associated metadata during the experiment editing phase, capturing details like the defined interval and whether to retain a value after the defined interval.
//!
//! ## Utilities:
//!
//! - The `InstrArgs` type alias provides a convenient way to define instruction arguments using a dictionary with string keys and float values.
//!
//! - The module makes use of the `maplit` crate to enable easy creation of IndexMaps.
//!
//! ## Features:
//!
//! - Easy creation of instruction objects with utility methods such as `new_const` and `new_sine`.
//! - Ability to evaluate instructions and in-place populate given time array views with the resulting float-point values.
//! - Support for default values in instructions, allowing for flexibility and ease of use.

use std::cmp::Ordering;
use indexmap::IndexMap;
use std::f64::consts::PI;
use std::fmt;
use ndarray::array;

/// Type alias for instruction arguments: a dictionary with key-value pairs of
/// string (argument name) and float (value)
pub type InstrArgs = IndexMap<String, f64>;

/// Enum type for different instructions. Supported instructions: `CONST`, `SINE`
#[derive(Clone, PartialEq)]
pub enum InstrType {
    CONST,
    SINE,
    LINRAMP, // Linear ramp
}
impl fmt::Display for InstrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InstrType::CONST => "CONST",
                InstrType::SINE => "SINE",
                InstrType::LINRAMP => "LINRAMP",
            }
        )
    }
}

// / This function uses [`other_function`] to ...
// /
// / [`other_function`]: ./path/to/other/function

// Instruction struct consists of instr_type (enumerated type) and argument dictionary
/// Struct for a general instruction, consisting of type and arguments.
///
/// Different instruction types expects different fields in their argument dictionary.
/// Behavior for minimally expected keys are defined in `Instruction::new`, behavior of
/// default values are defined in `Instruction::eval_inplace`.
///
/// ## Implemented instruction types and their expected fields:
/// 1. `InstrType::CONST`:
///    - `const`
/// 2. `InstrType::SINE`:
///    - `freq`
///    - `amplitude`: Default is `1.0`
///    - `offset`: Default is `0.0`
///    - `phase`: Default is `0.0`
/// 3. `InstrType::LINRAMP`: 
///     - `start_val`
///     - `end_val`
#[derive(Clone, PartialEq)]
pub struct Instruction {
    pub instr_type: InstrType,
    pub args: InstrArgs,
}
impl Instruction {
    /// Constructs an `Instruction` object.
    ///
    /// This method serves as the foundational constructor upon which custom constructor
    /// wrappers for new instructions should be built. For each instruction type,
    /// it ensures that the `args` dictionary contains the required keys.
    ///
    /// Missing keys will cause a panic.
    ///
    /// # Examples
    ///
    /// Constructing a new `CONST` instruction
    /// (this is effectively the underlying implementation for [`Instruction::new_const`],
    /// the more convenient constructor):
    ///
    /// ```
    /// use nicompiler_backend::instruction::*;
    /// use indexmap::IndexMap;;
    ///
    /// let mut const_args = InstrArgs::new();
    /// const_args.insert("value".to_string(), 1.0);
    /// let const_instr = Instruction::new(InstrType::CONST, const_args);
    /// ```
    ///
    /// If you fail to provide the required argument fields, it will panic:
    ///
    /// ```should_panic
    /// # use nicompiler_backend::instruction::*;
    /// # use indexmap::IndexMap;;
    /// let mut const_args = InstrArgs::new();
    /// let const_instr = Instruction::new(InstrType::CONST, const_args);
    /// ```
    ///
    /// The panic message will be:
    /// `thread 'main' panicked at 'Expected instr type CONST to contain key value'`.
    ///
    /// Constructing a new `SINE` instruction:
    ///
    /// ```
    /// # use nicompiler_backend::instruction::*;
    /// # use indexmap::IndexMap;;

    /// let mut sine_args = InstrArgs::new();
    /// sine_args.insert("freq".to_string(), 10.0);
    /// sine_args.insert("offset".to_string(), 1.0); // amplitude and phase will use default values
    /// let sine_instr = Instruction::new(InstrType::SINE, sine_args);
    /// ```
    pub fn new(instr_type: InstrType, args: InstrArgs) -> Self {
        let panic_no_key = |keys: &[&str]| {
            for &key in keys {
                if !args.contains_key(key) {
                    panic!("Expected instr type {} to contain key {}", instr_type, key)
                }
            }
        };
        match instr_type {
            InstrType::CONST => panic_no_key(&["value"]),
            InstrType::SINE => panic_no_key(&["freq"]),
            InstrType::LINRAMP => panic_no_key(&["start_val", "end_val", "start_time", "end_time"]),
        };
        Instruction { instr_type, args }
    }

    /// Evaluates the instruction and populates the given array view with float-point values.
    ///
    /// This method takes a mutable array view (`t_arr`) and modifies its values in-place based on the instruction type and its arguments.
    ///
    /// - For `InstrType::CONST`, the array will be filled with the constant value specified by the `value` argument.
    /// - For `InstrType::SINE`, a sinusoidal waveform is generated using the arguments `freq`, `amplitude`, `offset`, and `phase`. Default values are used if certain arguments are not provided.
    ///
    /// # Arguments
    ///
    /// * `t_arr` - A mutable 1D array view that will be populated with the evaluated values.
    ///
    /// # Examples
    ///
    /// Given an instruction set with a constant and a sine instruction, and an array representing time values from 0 to 1:
    ///
    /// ```
    /// use ndarray::{Array2, Array1};
    /// use nicompiler_backend::instruction::*;
    ///
    /// let t_row = ndarray::Array1::linspace(0.0, 1.0, 10);
    /// let mut t_values = ndarray::stack(ndarray::Axis(0), &[t_row.view(), t_row.view()]).unwrap();
    /// // Use wrappers to create sine and constant instructions same as the examples above
    /// let const_instr = Instruction::new_const(1.0);
    /// const_instr.eval_inplace(&mut t_values.row_mut(0));
    ///
    /// let sine_instr = Instruction::new_sine(10.0, None, None, Some(1.0));
    /// sine_instr.eval_inplace(&mut t_values.row_mut(1));
    /// assert!(t_values[[0, 0]] == 1. && t_values[[0, 1]] == 1.);
    /// ```
    pub fn eval_inplace(&self, t_arr: &mut ndarray::ArrayViewMut1<f64>) {
        // Tocheck: t_arr may not be 1-dimensional. 
        match self.instr_type {
            InstrType::CONST => {
                let value = *self.args.get("value").unwrap();
                t_arr.fill(value);
            }
            InstrType::SINE => {
                let freq = *self.args.get("freq").unwrap();
                // Default values can be set by default with unwrap_or
                let amplitude = *self.args.get("amplitude").unwrap_or(&1.0);
                let offset = *self.args.get("offset").unwrap_or(&0.0);
                let phase = *self.args.get("phase").unwrap_or(&0.0);

                t_arr.map_inplace(|t| {
                    *t = (2.0 * PI * freq * (*t) + phase).sin() * amplitude + offset
                });
            }
            InstrType::LINRAMP => {
                let start_val = *self.args.get("start_val").unwrap();
                let end_val = *self.args.get("end_val").unwrap();
                let t_start = *self.args.get("start_time").unwrap();
                let t_end = *self.args.get("end_time").unwrap();
                // println!("{:?} \n {:?}", t_arr.shape(), t_arr);

                t_arr.map_inplace(|t| {
                    *t = (*t - t_start) * (end_val - start_val) / (t_end - t_start) + start_val;
                });
            }
        }
    }

    /// Evaluate function at a single time point
    pub fn eval_point(&self, t: f64) -> f64 {
        let mut t_arr = array![t];
        self.eval_inplace(&mut t_arr.view_mut());
        t_arr[0]
    }

    /// Wrapper for conveniently creating new constant instructions.
    /// Example usage equivalent to the constant example above:
    /// ```
    /// # use nicompiler_backend::instruction::*;
    /// let const_instr = Instruction::new_const(1.0);
    /// ```
    pub fn new_const(value: f64) -> Instruction {
        let mut args = IndexMap::new();
        args.insert(String::from("value"), value);
        Instruction::new(InstrType::CONST, args)
    }

    /// Wrapper for conveniently creating new linear ramp instructions. 
    /// `start_val` will be the value on the first tick, and `end_val` value on the last tick. 
    pub fn new_linramp(start_val: f64, end_val:f64, start_time: f64, end_time: f64) -> Instruction {
        let mut args = IndexMap::new();
        args.insert(String::from("start_val"), start_val);
        args.insert(String::from("end_val"), end_val);
        args.insert(String::from("start_time"), start_time);
        args.insert(String::from("end_time"),end_time);
        Instruction::new(InstrType::LINRAMP, args)
    }

    /// Constructs a new sine instruction with provided parameters.
    ///
    /// Allows for convenient creation of sine instructions by specifying the frequency and optionally, amplitude, phase, and DC offset. Unspecified parameters will not be included in the instruction's argument dictionary, allowing for default values to be used elsewhere if necessary.
    ///
    /// # Arguments
    ///
    /// - `freq`: The frequency of the sine wave.
    /// - `amplitude`: Optional amplitude of the sine wave. If `None`, it will not be set in the instruction.
    /// - `phase`: Optional phase offset of the sine wave in radians. If `None`, it will not be set in the instruction.
    /// - `dc_offset`: Optional DC offset for the sine wave. If `None`, it will not be set in the instruction.
    ///
    /// # Examples
    ///
    /// Constructing a sine instruction with a specified frequency, and DC offset. Amplitude and phase will use any default values defined elsewhere:
    ///
    /// ---
    /// # use nicompiler_backend::instruction::*;
    ///
    /// let sine_instr = Instruction::new_sine(10.0, None, None, Some(1.0));
    /// ---
    ///
    pub fn new_sine(
        freq: f64,
        amplitude: Option<f64>,
        phase: Option<f64>,
        dc_offset: Option<f64>,
    ) -> Instruction {
        let mut instr_args = IndexMap::new();
        instr_args.insert(String::from("freq"), freq);
        // For each optional argument, if specified, insert into dictionary
        [
            ("amplitude", amplitude),
            ("phase", phase),
            ("offset", dc_offset),
        ]
        .iter()
        .for_each(|(key, opt_value)| {
            if let Some(value) = *opt_value {
                instr_args.insert(key.to_string(), value);
            }
        });
        Instruction::new(InstrType::SINE, instr_args)
    }
}
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let args_string = self
            .args
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "[{}, {{{}}}]", self.instr_type, args_string)
    }
}

/// Struct containing function and start/end edge data of the instruction.
///
/// # Fields:
/// - `instr` - the function struct
///
/// - `start_pos` - beginning of the instruction interval
///
/// - `end_spec` specifies instruction interval end. Can be `Some` or `None`:
///     - `Some((end_pos, keep_val))` - instruction has specific `end_pos`.
///       If there is a gap until the next edge (the next instruction or global end), compiler will keep a constant value starting at `end_pos`.
///       If `keep_val` is `true`, it will be the last instruction value, otherwise it will be the channel default.
///
///     - `None` - no specified end, instruction will span until the next edge (start of the next instruction or global end).
///
/// # Edge inclusion:
/// - `start_pos` is *inclusive*, sample for `start_pos` clock tick is covered;
/// - `end_pos` is *exclusive*, sample for `end_pos` clock tick is not covered, the next instruction can start here otherwise it will be covered by padding;
///
/// # Minimal instruction length is 1 clock tick:
/// - If `end_spec` is `Some`, minimal `end_pos` is `start_pos + 1`
/// - If `end_spec` is `None`, the next instruction must start no earlier than `start_pos + 1`
///
/// # Ordering
/// `InstrBook` implements ordering based on `start_pos` to facilitate sorting.
///
pub struct InstrBook {
    pub start_pos: usize,
    pub end_spec: Option<(usize, bool)>,
    pub instr: Instruction,
}
impl InstrBook {
    /// Constructs a new `InstrBook` object.
    ///
    /// Checks that `end_pos` is strictly greater than `start_pos`.
    ///
    /// # Arguments
    /// - `start_pos`: Starting position (inclusive).
    /// - `end_spec`: specifies instruction interval end. Can be `Some` or `None`:
    ///     - `Some((end_pos, keep_val))` - instruction has specific `end_pos`.
    ///       If there is a gap until the next edge (the next instruction or global end), compiler will keep a constant value starting at `end_pos`.
    ///       If `keep_val` is `true`, it will be the last instruction value, otherwise it will be the channel default.
    ///     - `None` - no specified end, instruction will span until the next edge (start of the next instruction or global end).
    /// - `func`: The associated function.
    ///
    /// # Examples
    ///
    /// Constructing a valid `InstrBook`:
    ///
    /// ```
    /// # use nicompiler_backend::instruction::*;
    /// let instruction = Instruction::new(InstrType::CONST, [("value".to_string(), 1.0)].iter().cloned().collect());
    /// let book = InstrBook::new(0, Some((5, true)), instruction);
    /// ```
    ///
    /// Attempting to construct an `InstrBook` with `end_pos` not greater than `start_pos` will panic:
    ///
    /// ```should_panic
    /// # use nicompiler_backend::instruction::*;
    /// let instruction = Instruction::new(InstrType::CONST, [("value".to_string(), 1.0)].iter().cloned().collect());
    /// let book = InstrBook::new(5, Some((5, true)), instruction);
    /// ```
    ///
    /// The panic message will be:
    /// `Instruction { /* ... */ } end_pos 5 should be strictly greater than start_pos 5`.
    pub fn new(start_pos: usize, end_spec: Option<(usize, bool)>, func: Instruction) -> Self {
        if let Some((end_pos, _keep_val)) = &end_spec {
            // Sanity check - the smallest permissible instruction length is 1 tick
            assert!(
                start_pos + 1 <= *end_pos,
                "Instruction must satisfy `start_pos + 1 <= end_pos` \n\
                 However, provided instruction has start_pos = {} and end_pos = {}",
                start_pos, end_pos
            )
        }
        InstrBook {
            start_pos,
            end_spec,
            instr: func,
        }
    }
    /// Returns the value of the `end_pos` field
    pub fn end_pos(&self) -> Option<usize> {
        match self.end_spec {
            Some((end_pos, _keep_val)) => Some(end_pos),
            None => None,
        }
    }
    /// "Effective" end position
    ///
    /// If `Self.end_spec` is `Some`, simply returns `end_pos`.
    ///
    /// If `Self.end_spec` is `None`, returns `(start_pos + 1)`.
    /// This is because "go-something" instruction must have at least one tick - `start_pos` - to have any effect.
    /// So the "effective end", the earliest time any subsequent instruction can be starting at is `(start_pos + 1)`.
    pub fn eff_end_pos(&self) -> usize {
        // "go_something"-type instruction don't have a specific end_pos
        // but must have space for at least one tick to have any effect,
        // so the closest permissible end_pos is (start_pos + 1)
        match self.end_pos() {
            Some(end_pos) => end_pos,
            None => self.start_pos + 1,
        }
    }
    /// Returns `Some(end_pos - start_pos)` or `None` if not specified
    pub fn dur(&self) -> Option<usize> {
        match self.end_spec {
            Some((end_pos, _keep_val)) => Some(end_pos - self.start_pos),
            None => None,
        }
    }
}
// Support total ordering for InstrBook
impl Ord for InstrBook {
    fn cmp(&self, other: &Self) -> Ordering {
        // We reverse the order to make BinaryHeap a min-heap based on start_pos
        self.start_pos.cmp(&other.start_pos)
    }
}
impl PartialOrd for InstrBook {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for InstrBook {
    fn eq(&self, other: &Self) -> bool {
        self.start_pos == other.start_pos
    }
}
impl fmt::Display for InstrBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let end_spec = match self.end_spec {
            Some((end_pos, keep_val)) => format!("end_pos={end_pos}, keep_val={keep_val}"),
            None => "no specified end".to_string(),
        };
        write!(
            f,
            "InstrBook({}, start_pos={}, {})",
            self.instr, self.start_pos, end_spec
        )
    }
}
impl Eq for InstrBook {}
