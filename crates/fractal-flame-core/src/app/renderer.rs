use crate::domain::transformation::Transformation;
use crate::domain::{Color, FractalImage, Pixel, Point, Rect};
use crate::infra::random;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct Renderer {
    pub canvas: Arc<FractalImage>,
    pub world: Arc<Rect>,
    pub transformations: Arc<Vec<Box<dyn Transformation + Send + Sync>>>,
    pub samples: usize,
    pub iter_per_sample: usize,
    pub symmetry: usize,
    pub gamma: f64,
    pub max_threads: usize,
    pub progress: Option<Arc<AtomicUsize>>,
}

impl Renderer {
    pub fn new(
        canvas: FractalImage,
        world: Rect,
        transformations: Vec<Box<dyn Transformation + Send + Sync>>,
        samples: usize,
        iter_per_sample: usize,
        symmetry: usize,
        gamma: f64,
        max_threads: usize,
    ) -> Self {
        Self {
            canvas: Arc::new(canvas),
            world: Arc::new(world),
            transformations: Arc::new(transformations),
            samples,
            iter_per_sample,
            symmetry,
            gamma,
            max_threads,
            progress: None,
        }
    }

    fn render_sample(
        &self,
        thread_id: usize,
        samples_per_thread: usize,
        remainder: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_sample = thread_id * samples_per_thread
            + if thread_id < remainder {
                thread_id
            } else {
                remainder
            };
        let end_sample =
            start_sample + samples_per_thread + if thread_id < remainder { 1 } else { 0 };

        for _ in start_sample..end_sample {
            let start_point = get_random_point_from_world(&self.world)?;
            let mut current_point = start_point;

            for iter in -20i32..self.iter_per_sample as i32 {
                let transformation = get_random_transformation(&self.transformations)?;
                current_point = transformation.apply(&current_point);

                if iter < 0 {
                    continue;
                }

                for symmetry_step in 0..self.symmetry {
                    let theta = (symmetry_step as f64)
                        * (2.0 * std::f64::consts::PI / self.symmetry as f64);
                    let symmetry_transform =
                        crate::app::transformations::symmetry::Symmetry::new(theta);
                    let symmetric_point = symmetry_transform.apply(&current_point);

                    if !self.world.contains_point(&symmetric_point) {
                        continue;
                    }

                    if let Some(pixel) = self.map_to_pixel(&symmetric_point) {
                        let color = transformation.color();
                        calculate_color(pixel, color);
                    }
                }
            }

            if let Some(ref progress) = self.progress {
                progress.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    }

    pub fn render(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let samples_per_thread = self.samples / self.max_threads;
        let remainder = self.samples % self.max_threads;

        (0..self.max_threads)
            .into_par_iter()
            .with_min_len(1)
            .try_for_each(|thread_id| {
                self.render_sample(thread_id, samples_per_thread, remainder)
            })?;

        Ok(())
    }

    pub fn apply_gamma_correction(&self) {
        let mut max_normal = 0.0f64;

        for y in 0..self.canvas.height {
            for x in 0..self.canvas.width {
                if let Some(pixel) = self.canvas.pixel_at(x, y) {
                    if let Ok(mut data) = pixel.write() {
                        if data.hit_count > 0 {
                            data.normal = (data.hit_count as f64).log10();
                            if data.normal > max_normal {
                                max_normal = data.normal;
                            }
                        }
                    }
                }
            }
        }

        if max_normal > 0.0 {
            for y in 0..self.canvas.height {
                for x in 0..self.canvas.width {
                    if let Some(pixel) = self.canvas.pixel_at(x, y) {
                        if let Ok(mut data) = pixel.write() {
                            if data.hit_count > 0 {
                                data.normal /= max_normal;

                                let gamma_factor = data.normal.powf(1.0 / self.gamma);
                                data.color.r = ((data.color.r as f64) * gamma_factor) as u8;
                                data.color.g = ((data.color.g as f64) * gamma_factor) as u8;
                                data.color.b = ((data.color.b as f64) * gamma_factor) as u8;
                            }
                        }
                    }
                }
            }
        }
    }

    fn map_to_pixel(&self, point: &Point) -> Option<&Pixel> {
        let x = ((self.canvas.width as f64) * (point.x - self.world.x) / self.world.width) as usize;
        let y =
            ((self.canvas.height as f64) * (point.y - self.world.y) / self.world.height) as usize;

        self.canvas.pixel_at(x, y)
    }
}

fn get_random_point_from_world(
    world: &Rect,
) -> Result<Point, Box<dyn std::error::Error + Send + Sync>> {
    let x = random::generate_f64(world.x, world.x + world.width, false)?;
    let y = random::generate_f64(world.y, world.y + world.height, false)?;
    Ok(Point::new(x, y))
}

fn get_random_transformation(
    transformations: &[Box<dyn Transformation + Send + Sync>],
) -> Result<&Box<dyn Transformation + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
    if transformations.is_empty() {
        return Err("No transformations available".into());
    }

    let total_weight: f64 = transformations.iter().map(|t| t.weight()).sum();

    let random_value = random::generate_f64(0.0, total_weight, true)?;

    let mut current_weight = 0.0;
    for transformation in transformations {
        current_weight += transformation.weight();
        if random_value <= current_weight {
            return Ok(transformation);
        }
    }

    Ok(&transformations[transformations.len() - 1])
}

pub fn get_random_color() -> Result<Color, Box<dyn std::error::Error + Send + Sync>> {
    let r = random::generate_i32(0, 256)?;
    let g = random::generate_i32(0, 256)?;
    let b = random::generate_i32(0, 256)?;
    Ok(Color {
        r: r as u8,
        g: g as u8,
        b: b as u8,
    })
}

fn calculate_color(pixel: &Pixel, color: &Color) {
    if let Ok(mut data) = pixel.write() {
        if data.hit_count == 0 {
            data.color = *color;
        } else {
            data.color.r = ((data.color.r as u16 + color.r as u16) / 2) as u8;
            data.color.g = ((data.color.g as u16 + color.g as u16) / 2) as u8;
            data.color.b = ((data.color.b as u16 + color.b as u16) / 2) as u8;
        }
        data.hit_count += 1;
    }
}
