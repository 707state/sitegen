use std::f64::consts::{PI, TAU};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

// ─── 确定性 PRNG（xorshift64） ─────────────────────────────────────────────────

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0xdeadbeef } else { seed })
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// [0, 1)
    fn f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// [lo, hi)
    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.f64() * (hi - lo)
    }

    /// [0, n)
    fn usize_n(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// 正态分布近似（Box-Muller）
    fn normal(&mut self) -> f64 {
        let u = self.f64().max(1e-10);
        let v = self.f64();
        (-2.0 * u.ln()).sqrt() * (TAU * v).cos()
    }
}

// ─── FNV-1a 哈希 ───────────────────────────────────────────────────────────────

fn fnv1a(s: &str) -> u64 {
    const BASIS: u64 = 14695981039346656037;
    const PRIME: u64 = 1099511628211;
    let mut hash = BASIS;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

// ─── 调色板（12 套，比原来翻倍） ───────────────────────────────────────────────

/// (背景色, [前景色列表])
const PALETTES: &[(&str, &[&str])] = &[
    // 深夜蓝
    (
        "#060d1a",
        &["#0f3460", "#1a7abf", "#4db8ff", "#a8deff", "#2e5f8a"],
    ),
    // 暮光橙
    (
        "#1a0a00",
        &["#c45c00", "#f5a623", "#ffd580", "#ffedba", "#e07b39"],
    ),
    // 翡翠绿
    (
        "#001510",
        &["#066540", "#0fa86a", "#3be8a0", "#a3f5d8", "#1dc47c"],
    ),
    // 暗紫幻境
    (
        "#0a0014",
        &["#4a1080", "#8b3fcf", "#c49cf5", "#ecdeff", "#6e28b0"],
    ),
    // 珊瑚玫瑰
    (
        "#1a0010",
        &["#9c1246", "#e83d8a", "#f7a8cf", "#ffe0ef", "#c42e68"],
    ),
    // 钢铁蓝灰
    (
        "#080f18",
        &["#1e3a5f", "#2e6096", "#5b9bd5", "#b8d8f8", "#3d7abf"],
    ),
    // 极光绿紫
    (
        "#020d08",
        &["#00573f", "#00b877", "#7affd4", "#30cfae", "#9b59b6"],
    ),
    // 熔岩红
    (
        "#100500",
        &["#8b1a00", "#d43f00", "#ff7a2f", "#ffc49a", "#f05020"],
    ),
    // 银河黑金
    (
        "#050505",
        &["#7a6200", "#c4a400", "#f5d000", "#fff3a0", "#e8b800"],
    ),
    // 赛博青
    (
        "#00100f",
        &["#006b60", "#00c4b4", "#4dfff0", "#b3fff9", "#00e8d8"],
    ),
    // 暮霞粉紫
    (
        "#12000e",
        &["#7a1060", "#c440a0", "#f58de0", "#ffd8f5", "#e060c0"],
    ),
    // 森林迷雾
    (
        "#040d06",
        &["#1a4a26", "#2e8045", "#60c87a", "#b8f0c8", "#42a85e"],
    ),
];

// ─── 颜色工具 ──────────────────────────────────────────────────────────────────

/// 将 "#rrggbb" 解析为 (r,g,b)
fn parse_hex(c: &str) -> (u8, u8, u8) {
    let c = c.trim_start_matches('#');
    let v = u32::from_str_radix(c, 16).unwrap_or(0x888888);
    ((v >> 16) as u8, (v >> 8 & 0xff) as u8, (v & 0xff) as u8)
}

fn rgba(r: u8, g: u8, b: u8, a: f64) -> String {
    format!("rgba({r},{g},{b},{a:.3})")
}

fn hex_to_rgba(hex: &str, a: f64) -> String {
    let (r, g, b) = parse_hex(hex);
    rgba(r, g, b, a)
}

// ─── 风格说明（8 种，通过 style_id 索引分发） ────────────────────────────────
// 0: Voronoi plain  1: Voronoi outlined  2: LowPoly
// 3: Ripple  4: Nebula  5: FlowGrid  6: Crystal  7: Rays

// ─── 主入口 ────────────────────────────────────────────────────────────────────

pub fn draw_cover(canvas: &HtmlCanvasElement, key: &str) {
    let seed = fnv1a(key);
    let mut rng = Rng::new(seed);

    let w = canvas.width() as f64;
    let h = canvas.height() as f64;

    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|o| o.dyn_into().ok())
        .expect("2d context");

    // 选调色板
    let pal_idx = rng.usize_n(PALETTES.len());
    let (bg, fg_slice) = PALETTES[pal_idx];
    let colors: Vec<&str> = fg_slice.to_vec();

    // 填背景
    ctx.set_fill_style_str(bg);
    ctx.fill_rect(0.0, 0.0, w, h);

    // 选风格（8 种）
    let style_id = rng.usize_n(8);
    match style_id {
        0 => draw_voronoi(&ctx, &mut rng, &colors, bg, w, h, false),
        1 => draw_voronoi(&ctx, &mut rng, &colors, bg, w, h, true),
        2 => draw_low_poly(&ctx, &mut rng, &colors, w, h),
        3 => draw_ripple(&ctx, &mut rng, &colors, bg, w, h),
        4 => draw_nebula(&ctx, &mut rng, &colors, w, h),
        5 => draw_flow_grid(&ctx, &mut rng, &colors, w, h),
        6 => draw_crystal(&ctx, &mut rng, &colors, w, h),
        7 => draw_rays(&ctx, &mut rng, &colors, bg, w, h),
        _ => draw_circuit(&ctx, &mut rng, &colors, w, h),
    }

    // 所有风格都叠加一层中心高光（增加立体感）
    overlay_vignette(&ctx, w, h);
}

// ─── 工具：最近邻 ──────────────────────────────────────────────────────────────

fn nearest(p: (f64, f64), sites: &[(f64, f64)]) -> usize {
    sites
        .iter()
        .enumerate()
        .map(|(i, &(sx, sy))| {
            let dx = p.0 - sx;
            let dy = p.1 - sy;
            (i, dx * dx + dy * dy)
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0)
}

// ─── 1/2. Voronoi ─────────────────────────────────────────────────────────────

fn draw_voronoi(
    ctx: &CanvasRenderingContext2d,
    rng: &mut Rng,
    colors: &[&str],
    _bg: &str,
    w: f64,
    h: f64,
    outlined: bool,
) {
    let n = 10 + rng.usize_n(8);
    let sites: Vec<(f64, f64)> = (0..n)
        .map(|_| (rng.range(0.0, w), rng.range(0.0, h)))
        .collect();

    let iw = w as u32;
    let ih = h as u32;

    // 逐行扫描，批量 fillRect
    for row in 0..ih {
        let y = row as f64 + 0.5;
        let mut col = 0u32;
        while col < iw {
            let x = col as f64 + 0.5;
            let idx = nearest((x, y), &sites);
            let mut end = col + 1;
            while end < iw && nearest((end as f64 + 0.5, y), &sites) == idx {
                end += 1;
            }
            // 黄金比例映射透明度，使相邻区域有明暗对比
            let alpha = 0.45 + 0.45 * ((idx as f64 * 0.618033988) % 1.0);
            ctx.set_global_alpha(alpha);
            ctx.set_fill_style_str(colors[idx % colors.len()]);
            ctx.fill_rect(col as f64, row as f64, (end - col) as f64, 1.0);
            col = end;
        }
    }
    ctx.set_global_alpha(1.0);

    if outlined {
        // 画 Delaunay 近似连线
        ctx.set_stroke_style_str("rgba(255,255,255,0.18)");
        ctx.set_line_width(0.6);
        for (i, &(x1, y1)) in sites.iter().enumerate() {
            let mut dists: Vec<(f64, usize)> = sites
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(j, &(x2, y2))| ((x2 - x1).hypot(y2 - y1), j))
                .collect();
            dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            for (_, j) in dists.iter().take(3) {
                let (x2, y2) = sites[*j];
                ctx.begin_path();
                ctx.move_to(x1, y1);
                ctx.line_to(x2, y2);
                let _ = ctx.stroke();
            }
        }
        // 站点装饰点
        ctx.set_global_alpha(0.7);
        for (i, &(x, y)) in sites.iter().enumerate() {
            ctx.set_fill_style_str(colors[i % colors.len()]);
            ctx.begin_path();
            let _ = ctx.arc(x, y, 1.8, 0.0, TAU);
            ctx.fill();
        }
        ctx.set_global_alpha(1.0);
    }
}

// ─── 3. Low Poly ──────────────────────────────────────────────────────────────

fn draw_low_poly(ctx: &CanvasRenderingContext2d, rng: &mut Rng, colors: &[&str], w: f64, h: f64) {
    // 构建点集：边界角 + 边界中点 + 随机内部点
    let mut pts: Vec<(f64, f64)> = vec![
        (0.0, 0.0),
        (w, 0.0),
        (w, h),
        (0.0, h),
        (w * 0.5, 0.0),
        (w, h * 0.5),
        (w * 0.5, h),
        (0.0, h * 0.5),
    ];
    let n_inner = 10 + rng.usize_n(8);
    for _ in 0..n_inner {
        pts.push((rng.range(w * 0.05, w * 0.95), rng.range(h * 0.05, h * 0.95)));
    }

    let n = pts.len();
    let mut tri_count = 0usize;

    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let (ax, ay) = pts[i];
                let (bx, by) = pts[j];
                let (cx, cy) = pts[k];
                let area = ((bx - ax) * (cy - ay) - (cx - ax) * (by - ay)).abs() * 0.5;
                // 面积过滤：不能太小也不能太大
                let min_area = (w * h) * 0.004;
                let max_area = (w * h) * 0.22;
                if area < min_area || area > max_area {
                    continue;
                }
                let centx = (ax + bx + cx) / 3.0;
                let centy = (ay + by + cy) / 3.0;
                if centx < 0.0 || centx > w || centy < 0.0 || centy > h {
                    continue;
                }
                let color = colors[tri_count % colors.len()];
                let alpha = 0.35 + 0.5 * ((tri_count as f64 * 0.618033) % 1.0);
                ctx.set_global_alpha(alpha);
                ctx.set_fill_style_str(color);
                ctx.begin_path();
                ctx.move_to(ax, ay);
                ctx.line_to(bx, by);
                ctx.line_to(cx, cy);
                ctx.close_path();
                ctx.fill();

                ctx.set_global_alpha(alpha * 0.4);
                ctx.set_stroke_style_str("rgba(255,255,255,0.12)");
                ctx.set_line_width(0.5);
                let _ = ctx.stroke();

                tri_count += 1;
            }
        }
    }
    ctx.set_global_alpha(1.0);
}

// ─── 4. Ripple（同心圆波纹） ───────────────────────────────────────────────────

fn draw_ripple(
    ctx: &CanvasRenderingContext2d,
    rng: &mut Rng,
    colors: &[&str],
    bg: &str,
    w: f64,
    h: f64,
) {
    // 1-3 个波源
    let n_centers = 1 + rng.usize_n(3);
    let centers: Vec<(f64, f64)> = (0..n_centers)
        .map(|_| (rng.range(w * 0.2, w * 0.8), rng.range(h * 0.2, h * 0.8)))
        .collect();

    let max_r = w.hypot(h);
    let ring_w = rng.range(2.5, 6.0); // 每个圆环宽度
    let n_rings = (max_r / ring_w) as usize + 2;

    for ring in 0..n_rings {
        let t = ring as f64 / n_rings as f64; // [0,1]
        let color_idx = ring % colors.len();
        let alpha = (0.6 - t * 0.5).max(0.08);

        for &(cx, cy) in &centers {
            let r = ring as f64 * ring_w;
            ctx.set_stroke_style_str(&hex_to_rgba(colors[color_idx], alpha));
            ctx.set_line_width(ring_w * 0.55);
            ctx.begin_path();
            let _ = ctx.arc(cx, cy, r.max(0.1), 0.0, TAU);
            let _ = ctx.stroke();
        }
    }

    // 叠加径向渐变：中心亮
    for &(cx, cy) in &centers {
        let r = max_r * 0.6;
        let grad = ctx.create_radial_gradient(cx, cy, 0.0, cx, cy, r).unwrap();
        let (br, bg2, bb) = parse_hex(bg);
        let _ = grad.add_color_stop(0.0, &rgba(br, bg2, bb, 0.0));
        let _ = grad.add_color_stop(1.0, &rgba(br, bg2, bb, 0.55));
        ctx.set_fill_style_canvas_gradient(&grad);
        ctx.fill_rect(0.0, 0.0, w, h);
    }

    // 细线装饰：从中心向外放射
    let n_lines = 8 + rng.usize_n(8);
    for i in 0..n_lines {
        let angle = TAU * i as f64 / n_lines as f64 + rng.range(-0.1, 0.1);
        let &(cx, cy) = &centers[i % centers.len()];
        ctx.set_stroke_style_str(&hex_to_rgba(colors[i % colors.len()], 0.22));
        ctx.set_line_width(0.6);
        ctx.begin_path();
        ctx.move_to(cx, cy);
        ctx.line_to(cx + angle.cos() * max_r, cy + angle.sin() * max_r);
        let _ = ctx.stroke();
    }
}

// ─── 5. Nebula（星云粒子） ─────────────────────────────────────────────────────

fn draw_nebula(ctx: &CanvasRenderingContext2d, rng: &mut Rng, colors: &[&str], w: f64, h: f64) {
    // 几个"星云核"，每个核周围散布粒子
    let n_cores = 2 + rng.usize_n(3);
    let cores: Vec<(f64, f64, f64)> = (0..n_cores)
        .map(|_| {
            (
                rng.range(w * 0.15, w * 0.85),
                rng.range(h * 0.15, h * 0.85),
                rng.range(w * 0.18, w * 0.45), // 扩散半径
            )
        })
        .collect();

    // 大量散点（每个核 80-150 个粒子）
    let n_per_core = 80 + rng.usize_n(80);
    for (core_idx, &(cx, cy, spread)) in cores.iter().enumerate() {
        let base_color = colors[core_idx % colors.len()];
        for _ in 0..n_per_core {
            // 正态分布位置
            let px = cx + rng.normal() * spread * 0.5;
            let py = cy + rng.normal() * spread * 0.5;

            // 距核距离决定大小和透明度
            let dist = (px - cx).hypot(py - cy);
            let falloff = (-dist / spread).exp();
            let r = rng.range(0.8, 3.5) * falloff.powf(0.4);
            let alpha = rng.range(0.15, 0.85) * falloff;

            ctx.set_global_alpha(alpha);
            // 偶尔切换到相邻颜色
            let col = if rng.f64() < 0.25 {
                colors[(core_idx + 1) % colors.len()]
            } else {
                base_color
            };
            ctx.set_fill_style_str(col);
            ctx.begin_path();
            let _ = ctx.arc(px, py, r.max(0.3), 0.0, TAU);
            ctx.fill();
        }
    }

    // 几颗大"亮星"
    let n_stars = 3 + rng.usize_n(5);
    for i in 0..n_stars {
        let sx = rng.range(0.0, w);
        let sy = rng.range(0.0, h);
        let sr = rng.range(1.2, 2.8);
        ctx.set_global_alpha(0.9);
        ctx.set_fill_style_str(colors[i % colors.len()]);
        ctx.begin_path();
        let _ = ctx.arc(sx, sy, sr, 0.0, TAU);
        ctx.fill();

        // 十字光芒
        ctx.set_global_alpha(0.35);
        ctx.set_stroke_style_str(colors[i % colors.len()]);
        ctx.set_line_width(0.5);
        for &angle in &[0.0_f64, PI * 0.5] {
            ctx.begin_path();
            ctx.move_to(sx + angle.cos() * sr * 4.0, sy + angle.sin() * sr * 4.0);
            ctx.line_to(sx - angle.cos() * sr * 4.0, sy - angle.sin() * sr * 4.0);
            let _ = ctx.stroke();
        }
    }
    ctx.set_global_alpha(1.0);
}

// ─── 6. Flow Grid（流动网格） ──────────────────────────────────────────────────

fn draw_flow_grid(ctx: &CanvasRenderingContext2d, rng: &mut Rng, colors: &[&str], w: f64, h: f64) {
    let cols = 6 + rng.usize_n(5);
    let rows = 5 + rng.usize_n(4);
    let cell_w = w / cols as f64;
    let cell_h = h / rows as f64;

    // 随机波浪参数
    let freq_x = rng.range(0.8, 2.5);
    let freq_y = rng.range(0.8, 2.5);
    let phase = rng.range(0.0, TAU);
    let amp_x = rng.range(0.15, 0.4) * cell_w;
    let amp_y = rng.range(0.15, 0.4) * cell_h;

    let mut cell_idx = 0usize;
    for row in 0..rows {
        for col in 0..cols {
            let bx = col as f64 * cell_w;
            let by = row as f64 * cell_h;

            // 每格四个角加扰动
            let corner = |cx: f64, cy: f64| -> (f64, f64) {
                let nx = cx / w;
                let ny = cy / h;
                let dx = amp_x * (freq_x * ny * TAU + phase).sin();
                let dy = amp_y * (freq_y * nx * TAU + phase * 1.3).sin();
                (cx + dx, cy + dy)
            };

            let (x0, y0) = corner(bx, by);
            let (x1, y1) = corner(bx + cell_w, by);
            let (x2, y2) = corner(bx + cell_w, by + cell_h);
            let (x3, y3) = corner(bx, by + cell_h);

            let color = colors[cell_idx % colors.len()];
            let alpha = 0.3 + 0.55 * ((cell_idx as f64 * 0.618033) % 1.0);
            ctx.set_global_alpha(alpha);
            ctx.set_fill_style_str(color);
            ctx.begin_path();
            ctx.move_to(x0, y0);
            ctx.line_to(x1, y1);
            ctx.line_to(x2, y2);
            ctx.line_to(x3, y3);
            ctx.close_path();
            ctx.fill();

            // 格线
            ctx.set_global_alpha(0.12);
            ctx.set_stroke_style_str("rgba(255,255,255,0.9)");
            ctx.set_line_width(0.5);
            let _ = ctx.stroke();

            cell_idx += 1;
        }
    }
    ctx.set_global_alpha(1.0);
}

// ─── 7. Crystal（几何晶体） ────────────────────────────────────────────────────

fn draw_crystal(ctx: &CanvasRenderingContext2d, rng: &mut Rng, colors: &[&str], w: f64, h: f64) {
    let n_shards = 12 + rng.usize_n(12);
    for i in 0..n_shards {
        // 随机中心
        let cx = rng.range(0.0, w);
        let cy = rng.range(0.0, h);
        // 随机多边形边数（3-7）
        let sides = 3 + rng.usize_n(5);
        let r_outer = rng.range(w * 0.06, w * 0.32);
        let r_inner = r_outer * rng.range(0.35, 0.75); // 内半径（星形）
        let rot = rng.range(0.0, TAU);
        let is_star = rng.f64() < 0.45;

        ctx.begin_path();
        let point_count = if is_star { sides * 2 } else { sides };
        for p in 0..point_count {
            let angle = rot + TAU * p as f64 / point_count as f64;
            let r = if is_star && p % 2 == 1 {
                r_inner
            } else {
                r_outer
            };
            let px = cx + r * angle.cos();
            let py = cy + r * angle.sin();
            if p == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.close_path();

        // 填充
        let color = colors[i % colors.len()];
        let alpha = rng.range(0.25, 0.72);
        ctx.set_global_alpha(alpha);
        ctx.set_fill_style_str(color);
        ctx.fill();

        // 描边
        let border_color = colors[(i + 2) % colors.len()];
        ctx.set_global_alpha(alpha * 0.55);
        ctx.set_stroke_style_str(&hex_to_rgba(border_color, 0.9));
        ctx.set_line_width(rng.range(0.4, 1.2));
        let _ = ctx.stroke();

        // 内部高光线（从中心到某顶点）
        if rng.f64() < 0.5 {
            let highlight_angle = rot + rng.range(0.0, TAU);
            ctx.set_global_alpha(alpha * 0.3);
            ctx.set_stroke_style_str("rgba(255,255,255,0.8)");
            ctx.set_line_width(0.6);
            ctx.begin_path();
            ctx.move_to(cx, cy);
            ctx.line_to(
                cx + r_outer * highlight_angle.cos(),
                cy + r_outer * highlight_angle.sin(),
            );
            let _ = ctx.stroke();
        }
    }
    ctx.set_global_alpha(1.0);
}

// ─── 8. Rays（放射光束） ───────────────────────────────────────────────────────

fn draw_rays(
    ctx: &CanvasRenderingContext2d,
    rng: &mut Rng,
    colors: &[&str],
    bg: &str,
    w: f64,
    h: f64,
) {
    // 光源中心（偏向某个角落让构图更有张力）
    let cx = rng.range(w * 0.1, w * 0.9);
    let cy = rng.range(h * 0.1, h * 0.9);
    let max_r = w.hypot(h);

    let n_rays = 16 + rng.usize_n(24);
    let base_angle_step = TAU / n_rays as f64;

    for i in 0..n_rays {
        let angle_start = base_angle_step * i as f64 + rng.range(-0.04, 0.04);
        let angle_end = angle_start + base_angle_step * rng.range(0.3, 0.85);

        let color = colors[i % colors.len()];
        let alpha = rng.range(0.12, 0.55);

        // 扇形光束（近三角形）
        ctx.begin_path();
        ctx.move_to(cx, cy);
        // 圆弧近似：分段折线
        let steps = 4;
        for s in 0..=steps {
            let a = angle_start + (angle_end - angle_start) * s as f64 / steps as f64;
            ctx.line_to(cx + max_r * a.cos(), cy + max_r * a.sin());
        }
        ctx.close_path();

        ctx.set_global_alpha(alpha);
        ctx.set_fill_style_str(color);
        ctx.fill();
    }

    // 中心光晕（径向渐变）
    let halo_r = max_r * 0.45;
    if let Ok(grad) = ctx.create_radial_gradient(cx, cy, 0.0, cx, cy, halo_r) {
        let bright = colors[rng.usize_n(colors.len())];
        let _ = grad.add_color_stop(0.0, &hex_to_rgba(bright, 0.65));
        let _ = grad.add_color_stop(0.5, &hex_to_rgba(bright, 0.15));
        let _ = grad.add_color_stop(1.0, &hex_to_rgba(bright, 0.0));
        ctx.set_global_alpha(1.0);
        ctx.set_fill_style_canvas_gradient(&grad);
        ctx.fill_rect(0.0, 0.0, w, h);
    }

    // 背景角落压暗
    let (br, bg2, bb) = parse_hex(bg);
    for &(corner_x, corner_y) in &[(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)] {
        if let Ok(grad) =
            ctx.create_radial_gradient(corner_x, corner_y, 0.0, corner_x, corner_y, max_r * 0.6)
        {
            let _ = grad.add_color_stop(0.0, &rgba(br, bg2, bb, 0.4));
            let _ = grad.add_color_stop(1.0, &rgba(br, bg2, bb, 0.0));
            ctx.set_global_alpha(1.0);
            ctx.set_fill_style_canvas_gradient(&grad);
            ctx.fill_rect(0.0, 0.0, w, h);
        }
    }
    ctx.set_global_alpha(1.0);
}

// ─── 9. Circuit（电路板/迷宫） ────────────────────────────────────────────────

fn draw_circuit(ctx: &CanvasRenderingContext2d, rng: &mut Rng, colors: &[&str], w: f64, h: f64) {
    let grid = 6.0 + rng.range(0.0, 4.0); // 网格间距
    let line_w = rng.range(0.5, 1.2);

    // 随机游走生成电路线段
    let n_walks = 8 + rng.usize_n(10);
    for walk_i in 0..n_walks {
        let color = colors[walk_i % colors.len()];
        let alpha = rng.range(0.35, 0.8);
        ctx.set_stroke_style_str(&hex_to_rgba(color, alpha));
        ctx.set_line_width(line_w);
        ctx.set_global_alpha(1.0);

        // 起点对齐网格
        let mut x = (rng.range(0.0, w) / grid).floor() * grid;
        let mut y = (rng.range(0.0, h) / grid).floor() * grid;

        ctx.begin_path();
        ctx.move_to(x, y);

        let steps = 8 + rng.usize_n(12);
        for _ in 0..steps {
            // L 形走线（水平或垂直）
            let horiz = rng.f64() < 0.5;
            let dist = (1 + rng.usize_n(4)) as f64 * grid;
            let dir = if rng.f64() < 0.5 { 1.0 } else { -1.0 };

            if horiz {
                x = (x + dist * dir).clamp(0.0, w);
            } else {
                y = (y + dist * dir).clamp(0.0, h);
            }
            ctx.line_to(x, y);

            // 偶尔转弯前画一段横线
            if rng.f64() < 0.4 {
                let tx = (x + rng.range(-grid * 2.0, grid * 2.0)).clamp(0.0, w);
                ctx.line_to(tx, y);
                x = tx;
            }
        }
        let _ = ctx.stroke();

        // 节点圆点
        let n_nodes = 2 + rng.usize_n(3);
        ctx.set_fill_style_str(&hex_to_rgba(color, alpha + 0.1));
        for _ in 0..n_nodes {
            let nx = (rng.range(0.0, w) / grid).floor() * grid;
            let ny = (rng.range(0.0, h) / grid).floor() * grid;
            let nr = rng.range(1.2, 2.5);
            ctx.begin_path();
            let _ = ctx.arc(nx, ny, nr, 0.0, TAU);
            ctx.fill();

            // 环形装饰
            if rng.f64() < 0.5 {
                ctx.set_global_alpha(alpha * 0.45);
                ctx.begin_path();
                let _ = ctx.arc(nx, ny, nr * 2.5, 0.0, TAU);
                let _ = ctx.stroke();
                ctx.set_global_alpha(1.0);
            }
        }
    }

    // 整体叠加网格点阵（增强电路板质感）
    ctx.set_global_alpha(0.08);
    ctx.set_fill_style_str(colors[0]);
    let mut gx = 0.0;
    while gx <= w {
        let mut gy = 0.0;
        while gy <= h {
            ctx.begin_path();
            let _ = ctx.arc(gx, gy, 0.6, 0.0, TAU);
            ctx.fill();
            gy += grid;
        }
        gx += grid;
    }
    ctx.set_global_alpha(1.0);
}

// ─── 公共叠加层：暗角 + 中心高光 ──────────────────────────────────────────────

fn overlay_vignette(ctx: &CanvasRenderingContext2d, w: f64, h: f64) {
    // 暗角压边
    if let Ok(grad) =
        ctx.create_radial_gradient(w * 0.5, h * 0.5, 0.0, w * 0.5, h * 0.5, w.hypot(h) * 0.6)
    {
        let _ = grad.add_color_stop(0.0, "rgba(0,0,0,0)");
        let _ = grad.add_color_stop(0.7, "rgba(0,0,0,0)");
        let _ = grad.add_color_stop(1.0, "rgba(0,0,0,0.45)");
        ctx.set_fill_style_canvas_gradient(&grad);
        ctx.fill_rect(0.0, 0.0, w, h);
    }
    // 中心轻微高光（让图像更有立体感）
    if let Ok(grad) =
        ctx.create_radial_gradient(w * 0.5, h * 0.4, 0.0, w * 0.5, h * 0.4, w.max(h) * 0.55)
    {
        let _ = grad.add_color_stop(0.0, "rgba(255,255,255,0.06)");
        let _ = grad.add_color_stop(1.0, "rgba(255,255,255,0)");
        ctx.set_fill_style_canvas_gradient(&grad);
        ctx.fill_rect(0.0, 0.0, w, h);
    }
}

// ─── Yew 组件 ─────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct PostCoverProps {
    pub cover_key: String,
    #[prop_or(56)]
    pub width: u32,
    #[prop_or(56)]
    pub height: u32,
}

#[function_component(PostCover)]
pub fn post_cover(props: &PostCoverProps) -> Html {
    let canvas_ref = use_node_ref();
    let cover_key = props.cover_key.clone();
    let width = props.width;
    let height = props.height;

    {
        let canvas_ref = canvas_ref.clone();
        use_effect_with(cover_key.clone(), move |key| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                draw_cover(&canvas, key);
            }
            || ()
        });
    }

    html! {
        <canvas
            ref={canvas_ref}
            class="post-cover"
            width={width.to_string()}
            height={height.to_string()}
            aria-hidden="true"
        />
    }
}
