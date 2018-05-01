use Sample;

impl Sample {
    /// Converts a Sample into a Double-Precision Float Sample
    pub fn to_double_float(&self) -> Sample {
        match self {
            Sample::Unsigned8bits(v) => {
                Sample::DoubleFloat(((f64::from(v.clone())*2f64)/f64::from(<u8>::max_value()))-1f64)
            }
            Sample::Signed16bits(v) => {
                Sample::DoubleFloat(f64::from(v.clone())/f64::from(<i16>::max_value()))
            }
            Sample::Signed32bits(v) => {
                Sample::DoubleFloat(f64::from(v.clone())/f64::from(<i32>::max_value()))
            }
            _ => unimplemented!("No conversion to Double Float for this type")
        }
    }
}