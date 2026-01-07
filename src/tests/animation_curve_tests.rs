//! Unit tests for animation curve
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        animation_curve::AnimationCurve,
        devices::{AirConditioner, SimulationSettings},
    };

    #[test]
    fn test_clamp_forever_behaviour() {
        // TemperatureDeltaEfficiency.json has preWrapMode=8 and postWrapMode=8 (ClampForever)
        let json = include_str!("../curves/AirConditioner/TemperatureDeltaEfficiency.json");
        let curve = AnimationCurve::from_json(json).expect("parse");

        let first_t = -500.0;
        let last_t = 2000.0;

        // Below first -> clamp to first key value
        let v_below = curve.evaluate(first_t - 1000.0);
        let v_first = curve.evaluate(first_t);
        assert!((v_below - v_first).abs() < 1e-12);

        // Above last -> clamp to last key value
        let v_above = curve.evaluate(last_t + 500.0);
        let v_last = curve.evaluate(last_t);
        assert!((v_above - v_last).abs() < 1e-12);
    }

    #[test]
    fn test_pingpong_behaviour() {
        // OperationalTemperatureEfficiency.json has postWrapMode=4 (PingPong)
        let json = include_str!("../curves/AirConditioner/OperationalTemperatureEfficiency.json");
        let curve = AnimationCurve::from_json(json).expect("parse");

        let last = 1200.0;

        assert!((curve.evaluate(last + 100.0) - curve.evaluate(last - 100.0)).abs() < 1e-12);
        assert!((curve.evaluate(last + 200.0) - curve.evaluate(last - 200.0)).abs() < 1e-12);
        assert!((curve.evaluate(last + 1100.0) - curve.evaluate(last - 1100.0)).abs() < 1e-12);
    }

    #[test]
    fn curves_are_shared() {
        let a = AirConditioner::new(Some(SimulationSettings::default()));
        let b = AirConditioner::new(Some(SimulationSettings::default()));

        // Compare the arc pointers for curve equality
        let p1 = Arc::as_ptr(&a.borrow().get_temperature_delta_curve());
        let p2 = Arc::as_ptr(&b.borrow().get_temperature_delta_curve());
        assert_eq!(p1, p2, "Temperature delta curve should be shared");

        let q1 = Arc::as_ptr(&a.borrow().get_input_and_waste_curve());
        let q2 = Arc::as_ptr(&b.borrow().get_input_and_waste_curve());
        assert_eq!(q1, q2, "Input & waste curve should be shared");
    }
}
