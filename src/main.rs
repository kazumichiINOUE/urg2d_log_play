use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use minifb::{Key, Window, WindowOptions};

struct UrgDATA {
    data_type: String,
    time_stamp: i64,
    num: i32,
    start_angle: f32,
    end_angle: f32,
    step_angle: f32,
    echo_num: i32,
    data: Vec::<i64>,
}

impl UrgDATA {
    fn print_info(self) {
        println!("=====<{}:{}>=====", self.data_type, self.time_stamp);
        println!("num:{} start_a:{} end_a:{} step_a:{} echo:{}", 
                 self.num, self.start_angle, self.end_angle, self.step_angle, self.echo_num);
        println!("{:?}", self.data);
    }
}

fn main() {
    // データファイルのオープン
    let log_path = "urglog";
    let file = File::open(Path::new(&log_path)).expect("ログファイルのオープンに失敗しました");
    let reader = BufReader::new(file);

    let mut urg_data = Vec::<UrgDATA>::new();
    // ファイルからデータを読み込み，ベクトルへ格納
    for line in reader.lines() {
        if let Ok(line) = line {
            // 行を空白で分割
            let parts: Vec<&str> = line.split_whitespace().collect();

            let mut urg = UrgDATA {
                data_type:   parts[0].parse().expect("Typeのパースに失敗しました"),
                time_stamp:  parts[1].parse().expect("Timestampのパースに失敗しました"),
                num:         parts[2].parse().expect("numのパースに失敗しました"),
                start_angle: parts[3].parse().expect("start_angleのパースに失敗しました"),
                end_angle:   parts[4].parse().expect("end_angleのパースに失敗しました"),
                step_angle:  parts[5].parse().expect("step_angleのパースに失敗しました"),
                echo_num:    parts[6].parse().expect("echo_numのパースに失敗しました"),
                data: vec![],
            };
            for i in (7..urg.num).step_by(3) {
                urg.data.push(parts[i as usize].parse().expect("dataのパースに失敗しました"));
            }
            urg_data.push(urg);
        }
    }

    let (img_width, img_height) = (800, 800);
    let (img_origin_x, img_origin_y) = (img_width/2, img_height/2);
    let csize = 0.05;

    let argb: u32 = 255 << 24 | 125 << 16 | 125 << 8 | 125;     // 背景色
    let black: u32 = 255 << 24 | 0 << 16 | 0 << 8 | 0;          // 黒

    // 背景となる画像の用意（XY軸，ロボット）
    let mut buffer_base: Vec<u32> = vec![argb; img_width as usize * img_height as usize]; 
    for i in 0..img_width {     // X軸
        buffer_base[(img_origin_y * img_width + i) as usize] = black;
    }
    for i in 0..img_height {    // Y軸
        buffer_base[(i * img_width + img_origin_x) as usize] = black;
    }
    draw_circle(&mut buffer_base, img_width, img_height, img_origin_x as usize, img_origin_y as usize, 
                (0.5/csize) as usize, hsv_to_rgb(0.0, 0.0, 0.0));

    // 画像を表示するウィンドウを作成
    let mut window = Window::new(
        "Image with Plots",
        img_width as usize,
        img_height as usize,
        WindowOptions::default(),
        )
        .expect("ウィンドウの作成に失敗しました");

    let mut buffer = buffer_base.clone();
    for urg in urg_data {
        //let mut points: Vec<(f32, f32)> = vec![];
        buffer = buffer_base.clone();   // 背景のみの画像を用意
        for (index, &dat) in urg.data.iter().enumerate() {
            if dat < 35000 {
                let th = (0.25 * index as f32 + urg.start_angle) * 3.1415/180.0;
                let cs = th.cos();
                let sn = th.sin();
                let x = (dat as f32)/1000.0 * cs;
                let y = (dat as f32)/1000.0 * sn;
                let cx = (( x/csize) as i32 + img_origin_x) as usize;
                let cy = ((-y/csize) as i32 + img_origin_y) as usize;
                //points.push((x, y));
                draw_circle_fill(&mut buffer, img_width, img_height, cx, cy, 1, hsv_to_rgb(100.0, 1.0, 1.0));
            }
        }
        // ウィンドウの更新
        window
            .update_with_buffer(&buffer, img_width as usize, img_height as usize)
            .expect("ウィンドウの更新に失敗しました");
    }

    // 終了待ち
    while !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, img_width as usize, img_height as usize)
            .expect("ウィンドウの更新に失敗しました");
    }
}

fn draw_circle_fill(buffer: &mut Vec<u32>, width: i32, height: i32, center_x: usize, center_y: usize, radius: usize, argb: u32) {
    // 描画範囲を評価する式
    let is_in_img = |x: i32, y: i32, width: i32, height: i32| -> bool {
        x >= 0 && x < (width as i32) && y >= 0 && y < (height as i32)
    };

    let cx: i32 = center_x as i32;
    let cy: i32 = center_y as i32;
    let r: i32 = radius as i32;

    for x in cx - r..=cx + r {
        for y in cy - r..=cy + r {
            let dx = x - cx;
            let dy = y - cy;
            let distance_squared = dx*dx + dy*dy;
            if distance_squared <= r * r {
                if is_in_img(x, y, width, height) {
                    buffer[(y as usize) * (width as usize) + (x as usize)] = argb;
                }
            }
        }
    }
}

fn draw_circle(buffer: &mut Vec<u32>, width: i32, height: i32, center_x: usize, center_y: usize, radius: usize, argb: u32) {
    // 描画範囲を評価する式
    let is_in_img = |x: i32, y: i32, width: i32, height: i32| -> bool {
        x >= 0 && x < (width as i32) && y >= 0 && y < (height as i32)
    };

    let cx: i32 = center_x as i32;
    let cy: i32 = center_y as i32;
    let r: i32 = radius as i32;

    for x in cx - r..=cx + r {
        for y in cy - r..=cy + r {
            let dx = x - cx;
            let dy = y - cy;
            let distance_squared = dx*dx + dy*dy;
            if distance_squared >= (r - 1) * (r - 1) && distance_squared <= r * r {
                if is_in_img(x, y, width, height) {
                    buffer[(y as usize) * (width as usize) + (x as usize)] = argb;
                }
            }
        }
    }
}


fn hsv_to_rgb(h: f32, s: f32, v: f32) -> u32 {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h {
        h if h <  60.0 => (  c,   x, 0.0),
        h if h < 120.0 => (  x,   c, 0.0),
        h if h < 180.0 => (0.0,   c,   x),
        h if h < 240.0 => (0.0,   x,   c),
        h if h < 300.0 => (  x, 0.0,   c),
                      _=> (  c, 0.0,   x),
    };

    // ARGBの順に32ビットで返す
    255 << 24 |(((r + m) * 255.0) as u32) << 16 | (((g + m) * 255.0) as u32) << 8 | (((b + m) * 255.0) as u32)
}
