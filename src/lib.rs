// fn encode(current: f64, previous: f64, factor: i64) -> str {
//     current = Math.round(current * factor);
//     previous = Math.round(previous * factor);
//     let coordinate = current - previous;
//     coordinate <<= 1;
//     if current - previous < 0 {
//         coordinate = ~coordinate;
//     }
//     let mut output = "";
//     while (coordinate >= 0x20) {
//         output += String.fromCharCode((0x20 | (coordinate & 0x1f)) + 63);
//         coordinate >>= 5;
//     }
//     output += String.fromCharCode(coordinate + 63);
//     return output;
// }

pub fn encodeCoordinates(coordinates: Vec<[f64; 2]>, precision: u32) -> String {
    if coordinates.len() == 0 {
        return "".to_string();
    }
    let base: i64 = 10;
    let factor = base.pow(precision);

    // let mut output = encode(coordinates[0][0], 0, factor) +
    //     encode(coordinates[0][1], 0, factor);
    "hi".to_string()

    // for i in 1..coordinates.length {
    //     var a = coordinates[i], b = coordinates[i - 1];
    //     output += encode(a[0], b[0], factor);
    //     output += encode(a[1], b[1], factor);
    // }

    // return output;
    // a + b
}

#[cfg(test)]
mod tests {

    use super::encodeCoordinates;

    #[test]
    fn it_works() {
        let coords = vec![[1.0, 2.0], [3.0, 4.0]];
        assert_eq!(encodeCoordinates(coords, 5), "")
    }
}
