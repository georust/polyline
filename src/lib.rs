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

pub fn encode_coordinates(coordinates: Vec<[f64; 2]>, precision: u32) -> String {
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

pub fn decode_polyline(str: String, precision: u32) -> Vec<[f64; 2]> {
    let mut index = 0;
    let mut lat: f64 = 0.0;
    let mut lng: f64 = 0.0;
    let mut coordinates = vec![];
    let base: i32 = 10;
    let factor: i32 = base.pow(precision);

    // Coordinates have variable length when encoded, so just keep
    // track of whether we've hit the end of the string. In each
    // loop iteration, a single coordinate is decoded.
    while index < str.len() {

        let mut shift = 0;
        let mut result = 0;

        let mut byte = 0;

        while {
            byte = (str.chars().nth(index).unwrap() as u64) - 63;
            index = index + 1;
            result |= (byte & 0x1f) << shift;
            shift += 5;
            byte >= 0x20
        } { }

        let latitude_change: f64 = if (result & 1) > 0 {
            !(result >> 1)
        } else {
            result >> 1
        } as f64;

        shift = 0;
        result = 0;

        while {
            byte = (str.chars().nth(index).unwrap() as u64) - 63;
            index = index + 1;
            result |= (byte & 0x1f) << shift;
            shift += 5;
            byte >= 0x20
        } { }

        let longitude_change: f64 = if (result & 1) > 0 {
            !(result >> 1)
        } else {
            result >> 1
        } as f64;

        lat += latitude_change;
        lng += longitude_change;

        coordinates.push([lat / factor as f64, lng / factor as f64]);
    }

    coordinates
}

#[cfg(test)]
mod tests {

    use super::encode_coordinates;
    use super::decode_polyline;

    #[test]
    fn it_works() {
        let encoded = "_ibE_seK_seK_seK";
        let coords = vec![[1.0, 2.0], [3.0, 4.0]];
        let coords2 = vec![[1.0, 2.0], [3.0, 4.0]];
        assert_eq!(encode_coordinates(coords, 5), encoded);
        assert_eq!(decode_polyline(encoded.to_string(), 5), coords2);
    }
}
