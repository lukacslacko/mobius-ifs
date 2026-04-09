mod params;
mod quaternion;
mod ifs;
mod mesh;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mobius-ifs-render", about = "High-resolution quaternion IFS renderer")]
struct Args {
    /// Path to fractal parameters JSON (exported from quaternion.html)
    params: PathBuf,

    /// Voxel grid resolution (side length of cube)
    #[arg(short = 'n', long, default_value = "512")]
    resolution: usize,

    /// Number of IFS iterations
    #[arg(short, long, default_value = "80")]
    iterations: usize,

    /// Output PLY file path
    #[arg(short, long, default_value = "fractal.ply")]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let params = params::FractalParams::load(&args.params)?;
    let n = args.resolution;

    let mem_gb = 2.0 * (n * n * n) as f64 * 12.0 / (1024.0 * 1024.0 * 1024.0);
    eprintln!("Resolution: {}³ ({:.1} GB for two grids)", n, mem_gb);
    eprintln!("Transforms: {}, num_deg: {}, den_deg: {}, normalize: {}",
        params.n_transforms, params.num_deg, params.den_deg, params.normalize);
    eprintln!("Color weight: {:.2}, threshold: {}%", params.color_weight, params.threshold_pct);

    // IFS iteration
    let pb = ProgressBar::new(args.iterations as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg} [{bar:40}] {pos}/{len} iterations ({eta} remaining)")
        .unwrap()
        .progress_chars("=> "));
    pb.set_message("Computing IFS");

    let grid = ifs::compute_ifs(&params, n, args.iterations, &pb);
    pb.finish_with_message("IFS complete");

    // Compute threshold
    let pct = params.threshold_pct as f32 / 100.0;
    let threshold = mesh::compute_threshold(&grid, pct);
    eprintln!("Threshold ({}th percentile): {:.6}", params.threshold_pct, threshold);

    // Marching cubes
    eprintln!("Running marching cubes...");
    let m = mesh::marching_cubes(&grid, threshold);
    eprintln!("Mesh: {} vertices, {} triangles", m.vertices.len(), m.triangles.len());

    // Write PLY
    eprintln!("Writing {}...", args.output.display());
    mesh::write_ply(&m, &args.output)?;

    let file_size = std::fs::metadata(&args.output)?.len();
    eprintln!("Done! Output: {} ({:.1} MB)", args.output.display(), file_size as f64 / (1024.0 * 1024.0));

    Ok(())
}
