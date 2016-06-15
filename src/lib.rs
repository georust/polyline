use std::char;

fn scale(n: f64, factor: i32) -> i64 {
    let scaled: f64 = n * (factor as f64);
    scaled.round() as i64
}

fn encode(current: f64, previous: f64, factor: i32) -> String {
    let mut coordinate = (scale(current, factor) - scale(previous, factor)) << 1;
    if (current - previous) < 0.0 {
        coordinate = !coordinate;
    }
    let mut output: String = "".to_string();
    while coordinate >= 0x20 {
        output.push(char::from_u32(((0x20 | (coordinate & 0x1f)) + 63) as u32).unwrap());
        coordinate >>= 5;
    }
    output.push(char::from_u32((coordinate  + 63) as u32).unwrap());
    output
}

pub fn encodeCoordinates(coordinates: Vec<[f64; 2]>, precision: u32) -> String {
    if coordinates.len() == 0 {
        return "".to_string();
    }
    let base: i32 = 10;
    let factor: i32 = base.pow(precision);

    let mut output = encode(coordinates[0][0], 0.0, factor) +
        &encode(coordinates[0][1], 0.0, factor);

    for i in 1..coordinates.len() {
        let a = coordinates[i];
        let b = coordinates[i - 1];
        output = output + &encode(a[0], b[0], factor);
        output = output + &encode(a[1], b[1], factor);
    }
    output
}

#[cfg(test)]
mod tests {

    use super::encodeCoordinates;

    #[test]
    fn it_works() {
        let coords = vec![[1.0, 2.0], [3.0, 4.0]];
        assert_eq!(encodeCoordinates(coords, 5), "_ibE_seK_seK_seK")
    }
}
