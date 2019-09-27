type RPoint = (f64, f64);
type SPoint = (usize, usize);
type Color = (u8, u8, u8);


#[derive(Debug)]
struct Camera {
    center : RPoint,
    height : f64,
    screen_size : SPoint,
}

impl Camera {
    fn width(&self) -> f64 {
        self.height * (self.screen_size.0 as f64) / (self.screen_size.1 as f64)
    }

    fn to_real(&self, sp: SPoint) -> RPoint {
        let x = sp.0;
        let y = self.screen_size.1 - sp.1;

        let centered_x = x - self.screen_size.0 / 2;
        let centered_y = y - self.screen_size.1 / 2;

        let scaled_x = centered_x as f64 * self.width() / (self.screen_size.0 as f64);
        let scaled_y = centered_y as f64 * self.height / (self.screen_size.1 as f64);

        (scaled_x, scaled_y)
    }

    fn to_screen(&self, rp: RPoint) -> SPoint {
        let centered_x = (rp.0 * (self.screen_size.0 as f64) / self.width()).round() as i32;
        let centered_y = (rp.1 * (self.screen_size.1 as f64) / self.height).round() as i32;

        let x = (centered_x + self.screen_size.0 as i32 / 2) as usize;
        let y = (centered_y + self.screen_size.1 as i32 / 2) as usize;

        (x, self.screen_size.1 - y)
    }
}


struct RecursiveSequence<F>
    where F: Fn(RPoint) -> RPoint {
    f: F,
    start: RPoint
}

impl<F> Iterator for RecursiveSequence<F>
    where F: Fn(RPoint) -> RPoint {

    type Item = RPoint;

    fn next(&mut self) -> Option<Self::Item> {
        self.start = (self.f)(self.start);

        Some(self.start)
    }
}


fn print_ppm(image : Vec<Color>, size : (usize, usize)) {
    assert_eq!(image.len(), size.0 * size.1);

    // ppm header
    println!("P3");  // specify RGB format
    println!("{} {}", size.0, size.1);  // image dims
    println!("255");  // color range from 0..256

    for y in 0..size.1 {
        for x in 0..size.0 {
            let color = image[y * size.0 + x];
            println!("{} {} {}", color.0, color.1, color.2);
        }
    }
}


fn mix(a: Color, b: Color, x: f64) -> Color {
    assert!(0.0 <= x && x <= 1.0);

//    (0..3usize).map(|i| (a[i] as f64 * (1.0 - x) + b[i] as f64 * x) as u8).collect()
    (
        (a.0 as f64 * (1.0 - x) + b.0 as f64 * x) as u8,
        (a.1 as f64 * (1.0 - x) + b.1 as f64 * x) as u8,
        (a.2 as f64 * (1.0 - x) + b.2 as f64 * x) as u8
    )
}


fn gradient(colors : Vec<Color>) -> [Color; 256]{
    if colors.is_empty() { return [(0,0,0); 256]; }
    else if colors.len() == 1 { return [colors[0]; 256]; }

    let mut out = [colors[0]; 256];

    let nb_segments = colors.len() - 1;
    let mut a = colors[0];
    let mut b = colors[1];
    let mut i_segment = 1;

    for i in 0..256 {
        let pos = i as f64 / 255.0;
        if pos > i_segment as f64 / (nb_segments as f64) {
            i_segment += 1;
            a = b;
            b = colors[i_segment];
        }
        let seg_pos = pos * (nb_segments as f64) - (i_segment as f64 - 1.0);
        out[i] = mix(a, b, seg_pos);
    }

    out
}


fn main() {
    let screen_size  = (1080, 1080);    // We want the full picture inside the image

    let a = -1.8;
    let b = 3.8;
    let c = 1.2;
    let d = -0.3;
    let nb_points = 35_000_000;
    let f = |(x, y)|
        (f64::sin(a * x) + c * f64::sin(a * y),
         f64::sin(b * y) + d * f64::sin(b * x));

    let camera = Camera {
        center: (0.0, 0.0),
        height: 3.0 + 2.0*c, // We want the full picture inside the image
        screen_size: screen_size,
    };

    // Generate the sequence
    let mut rseq = RecursiveSequence { f: f, start: (1.0, 1.0) };
    let mut data: Vec<usize> = vec![0; screen_size.0 * screen_size.1];

    rseq.take(nb_points)
        .map(|p| camera.to_screen(p))
        .for_each(|(x, y)| {
            data[y*screen_size.0 + x] += 1
        });

    // Translate number of points into color
    let grad = gradient(vec![
        (0, 0, 0),
        (58, 145, 112),
        (229, 214, 121),
        (232, 171, 46),
        (236, 49, 9),
    ]);
    let grad2 = gradient(vec![
        (18, 16, 10),
        (20, 112, 106),
        (14, 141, 130),
        (244, 157, 47),
        (239, 72, 25),
    ]);

    let image = data.iter()
        .map(|&x| x.min(255))
        .map(|x| grad2[x])
        .collect::<Vec<(u8, u8, u8)>>();

    // Print the image on stdout
    print_ppm(image, screen_size);



//    println!("{:?}", camera.to_real(camera.to_screen((0.0109, -0.9))));
}
