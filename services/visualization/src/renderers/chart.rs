use mirage_common::Error;
use std::collections::HashMap;
use plotters::prelude::*;
use plotters::style::full_palette::{BLUE, RED, GREEN, YELLOW, PURPLE};

pub trait ChartRenderer {
    fn render_timeline(
        &self,
        data: &[serde_json::Value],
        width: u32,
        height: u32,
        options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error>;
    
    fn render_bar(
        &self,
        data: &[serde_json::Value],
        width: u32,
        height: u32,
        options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error>;
    
    fn render_pie(
        &self,
        data: &[serde_json::Value],
        width: u32,
        height: u32,
        options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error>;
}

// SVG Chart Renderer
pub struct SvgChartRenderer;

impl SvgChartRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ChartRenderer for SvgChartRenderer {
    fn render_timeline(
        &self,
        data: &[serde_json::Value],
        width: u32,
        height: u32,
        options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        
        // Create SVG backend
        let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
        
        // Fill background
        root.fill(&WHITE)
            .map_err(|e| Error::Internal(format!("Failed to fill background: {}", e)))?;

        // Parse dates from data (this is simplified)
        let mut time_points: Vec<(chrono::DateTime<chrono::Utc>, String, String)> = Vec::new();
        for item in data {
            if let (Some(created_at), Some(entity_type), Some(value)) = (
                item["created_at"].as_str(),
                item["entity_type"].as_str(),
                item["value"].as_str(),
            ) {
                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(created_at) {
                    time_points.push((date.into(), entity_type.to_string(), value.to_string()));
                }
            }
        }
        
        // Sort by date
        time_points.sort_by(|a, b| a.0.cmp(&b.0));
        
        if time_points.is_empty() {
            // Draw "No Data" label if empty
            root.draw_text(
                "No timeline data available",
                &("sans-serif", 20),
                &BLACK,
                (width as i32 / 2, height as i32 / 2),
            ).map_err(|e| Error::Internal(format!("Failed to draw text: {}", e)))?;
        } else {
            // Calculate date range for the chart
            let earliest = time_points.first().unwrap().0;
            let latest = time_points.last().unwrap().0;
            
            // Add some padding to the date range
            let date_range = latest.timestamp() - earliest.timestamp();
            let padding = date_range.max(86400) / 10; // At least 1 day padding
            
            let start_time = earliest - chrono::Duration::seconds(padding);
            let end_time = latest + chrono::Duration::seconds(padding);
            
            // Create the chart
            let mut chart = ChartBuilder::on(&root)
                .margin(40)
                .caption("Entity Timeline", ("sans-serif", 30))
                .x_label_area_size(40)
                .y_label_area_size(60)
                .build_cartesian_2d(start_time..end_time, 0..time_points.len() as i32)
                .map_err(|e| Error::Internal(format!("Failed to build chart: {}", e)))?;
                
            chart.configure_mesh()
                .x_labels(8)
                .y_labels(0) // No y labels needed
                .x_label_formatter(&|x| x.to_rfc3339())
                .draw()
                .map_err(|e| Error::Internal(format!("Failed to draw mesh: {}", e)))?;
            
            // Draw points on the timeline
            for (i, (date, entity_type, value)) in time_points.iter().enumerate() {
                let y = i as i32;
                
                // Draw a circle at the date point
                let color = match entity_type.as_str() {
                    "domain" => &BLUE,
                    "ip" => &RED,
                    "email" => &GREEN,
                    _ => &YELLOW,
                };
                
                chart.draw_series(std::iter::once(Circle::new((*date, y), 5, color.filled())))
                    .map_err(|e| Error::Internal(format!("Failed to draw point: {}", e)))?
                    .label(entity_type)
                    .legend(move |(x, y)| Circle::new((x, y), 5, color.filled()));
                
                // Draw a label with the value
                chart.draw_series(std::iter::once(Text::new(
                    value.clone(), 
                    (*date, y + 1), 
                    ("sans-serif", 12).into_font().color(BLACK),
                )))
                .map_err(|e| Error::Internal(format!("Failed to draw label: {}", e)))?;
            }
            
            // Draw the legend
            chart.configure_series_labels()
                .background_style(&WHITE)
                .border_style(&BLACK)
                .draw()
                .map_err(|e| Error::Internal(format!("Failed to draw legend: {}", e)))?;
        }
        
        root.present().map_err(|e| Error::Internal(format!("Failed to render: {}", e)))?;
        
        Ok(buffer.into())
    }
    
    fn render_bar(
        &self,
        data: &[serde_json::Value],
        width: u32,
        height: u32,
        options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        
        // Create SVG backend
        let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
        
        root.fill(&WHITE)
            .map_err(|e| Error::Internal(format!("Failed to fill background: {}", e)))?;
            
        // Group data by entity_type
        let mut type_counts: HashMap<String, i32> = HashMap::new();
        
        for item in data {
            if let Some(entity_type) = item["entity_type"].as_str() {
                *type_counts.entry(entity_type.to_string()).or_insert(0) += 1;
            }
        }
        
        if type_counts.is_empty() {
            root.draw_text(
                "No bar chart data available",
                &("sans-serif", 20),
                &BLACK,
                (width as i32 / 2, height as i32 / 2),
            ).map_err(|e| Error::Internal(format!("Failed to draw text: {}", e)))?;
        } else {
            // Sort the data
            let mut type_data: Vec<(String, i32)> = type_counts.into_iter().collect();
            type_data.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count, descending
            
            let max_count = type_data.iter().map(|(_, count)| *count).max().unwrap_or(0);
            
            // Build the chart
            let mut chart = ChartBuilder::on(&root)
                .margin(40)
                .caption("Entity Types", ("sans-serif", 30))
                .x_label_area_size(40)
                .y_label_area_size(60)
                .build_cartesian_2d(
                    0..type_data.len(),
                    0..max_count + (max_count / 5) // Add 20% margin to top
                )
                .map_err(|e| Error::Internal(format!("Failed to build chart: {}", e)))?;
                
            chart.configure_mesh()
                .x_labels(type_data.len())
                .x_label_formatter(&|idx| {
                    if *idx < type_data.len() {
                        type_data[*idx].0.clone()
                    } else {
                        "".to_string()
                    }
                })
                .draw()
                .map_err(|e| Error::Internal(format!("Failed to draw mesh: {}", e)))?;
                
            // Define colors for the bars
            let colors = [&BLUE, &RED, &GREEN, &YELLOW, &PURPLE];
            
            // Draw the bars
            chart.draw_series(
                type_data.iter().enumerate().map(|(i, (_, count))| {
                    let color = colors[i % colors.len()];
                    Rectangle::new(
                        [(i, 0), (i+1, *count)],
                        color.filled(),
                    )
                })
            )
            .map_err(|e| Error::Internal(format!("Failed to draw bars: {}", e)))?;
        }
        
        root.present().map_err(|e| Error::Internal(format!("Failed to render: {}", e)))?;
        
        Ok(buffer.into())
    }
    
    fn render_pie(
        &self,
        data: &[serde_json::Value],
        width: u32,
        height: u32,
        options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        
        // Create SVG backend
        let root = SVGBackend::with_string(&mut buffer, (width, height)).into_drawing_area();
        
        root.fill(&WHITE)
            .map_err(|e| Error::Internal(format!("Failed to fill background: {}", e)))?;
            
        // Group data by entity_type
        let mut type_counts: HashMap<String, i32> = HashMap::new();
        
        for item in data {
            if let Some(entity_type) = item["entity_type"].as_str() {
                *type_counts.entry(entity_type.to_string()).or_insert(0) += 1;
            }
        }
        
        if type_counts.is_empty() {
            root.draw_text(
                "No pie chart data available",
                &("sans-serif", 20),
                &BLACK,
                (width as i32 / 2, height as i32 / 2),
            ).map_err(|e| Error::Internal(format!("Failed to draw text: {}", e)))?;
        } else {
            // Sort the data
            let mut type_data: Vec<(String, i32)> = type_counts.into_iter().collect();
            type_data.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count, descending
            
            let total: i32 = type_data.iter().map(|(_, count)| *count).sum();
            
            // Define colors for the pie slices
            let colors = [
                RGBColor(66, 133, 244),   // Blue
                RGBColor(234, 67, 53),    // Red
                RGBColor(52, 168, 83),    // Green  
                RGBColor(251, 188, 5),    // Yellow
                RGBColor(103, 58, 183),   // Purple
                RGBColor(0, 172, 193),    // Cyan
                RGBColor(255, 64, 129),   // Pink
            ];
            
            // Create a pie chart in the center
            let center = (width as i32 / 2, height as i32 / 2);
            let radius = std::cmp::min(width, height) as i32 / 3;
            
            let mut current_angle = 0.0;
            
            // Draw pie slices
            for (i, (label, count)) in type_data.iter().enumerate() {
                let color = colors[i % colors.len()];
                let ratio = *count as f64 / total as f64;
                let angle = ratio * 2.0 * std::f64::consts::PI;
                
                // Draw the slice
                let end_angle = current_angle + angle;
                
                // Create points for the slice
                let mut points = vec![(center.0, center.1)];
                
                // Add points along the arc
                let steps = 50;
                for step in 0..=steps {
                    let angle = current_angle + (angle * step as f64 / steps as f64);
                    let x = center.0 + (radius as f64 * angle.cos()) as i32;
                    let y = center.1 + (radius as f64 * angle.sin()) as i32;
                    points.push((x, y));
                }
                
                // Draw the slice as a polygon
                root.draw_series(std::iter::once(Polygon::new(
                    points,
                    color.filled(),
                )))
                .map_err(|e| Error::Internal(format!("Failed to draw pie slice: {}", e)))?;
                
                // Draw label line
                let mid_angle = current_angle + (angle / 2.0);
                let label_dist = radius as f64 * 1.3;
                let label_x = center.0 + (label_dist * mid_angle.cos()) as i32;
                let label_y = center.1 + (label_dist * mid_angle.sin()) as i32;
                
                root.draw_series(std::iter::once(PathElement::new(
                    vec![
                        (center.0 + (radius as f64 * 0.8 * mid_angle.cos()) as i32,
                         center.1 + (radius as f64 * 0.8 * mid_angle.sin()) as i32),
                        (label_x, label_y)
                    ],
                    color.stroke_width(1),
                )))
                .map_err(|e| Error::Internal(format!("Failed to draw label line: {}", e)))?;
                
                // Draw label text
                let percentage = (ratio * 100.0).round() as i32;
                let label_text = format!("{}: {}%", label, percentage);
                
                root.draw_series(std::iter::once(Text::new(
                    label_text,
                    (label_x, label_y),
                    ("sans-serif", 12).into_font().color(BLACK),
                )))
                .map_err(|e| Error::Internal(format!("Failed to draw label text: {}", e)))?;
                
                current_angle = end_angle;
            }
        }
        
        root.present().map_err(|e| Error::Internal(format!("Failed to render: {}", e)))?;
        
        Ok(buffer.into())
    }
}

// PNG Chart Renderer
pub struct PngChartRenderer;

impl PngChartRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ChartRenderer for PngChartRenderer {
    fn render_timeline(
        &self,
        data: &[serde_json::Value],
        width: u32,
        height: u32,
        options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        
        // Create bitmap backend
        let root = BitMapBackend::new(&mut buffer, (width, height)).into_drawing_area();
        
        // Use the same logic as SVG renderer, just with different backend
        root.fill(&WHITE)
            .map_err(|e| Error::Internal(format!("Failed to fill background: {}", e)))?;
            
        // Draw a placeholder for now
        root.draw_text(
            "Timeline Chart (PNG version)",
            &("sans-serif", 20),
            &BLACK,
            (width as i32 / 2, height as i32 / 2),
        ).map_err(|e| Error::Internal(format!("Failed to draw text: {}", e)))?;
        
        root.present().map_err(|e| Error::Internal(format!("Failed to render: {}", e)))?;
        
        Ok(buffer)
    }
    
    fn render_bar(
        &self,
        data: &[serde_json::Value],
        width: u32,
        height: u32,
        options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        
        // Create bitmap backend
        let root = BitMapBackend::new(&mut buffer, (width, height)).into_drawing_area();
        
        // Use the same logic as SVG renderer, just with different backend
        root.fill(&WHITE)
            .map_err(|e| Error::Internal(format!("Failed to fill background: {}", e)))?;
            
        // Draw a placeholder for now
        root.draw_text(
            "Bar Chart (PNG version)",
            &("sans-serif", 20),
            &BLACK,
            (width as i32 / 2, height as i32 / 2),
        ).map_err(|e| Error::Internal(format!("Failed to draw text: {}", e)))?;
        
        root.present().map_err(|e| Error::Internal(format!("Failed to render: {}", e)))?;
        
        Ok(buffer)
    }
    
    fn render_pie(
        &self,
        data: &[serde_json::Value],
        width: u32,
        height: u32,
        options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        
        // Create bitmap backend
        let root = BitMapBackend::new(&mut buffer, (width, height)).into_drawing_area();
        
        // Use the same logic as SVG renderer, just with different backend
        root.fill(&WHITE)
            .map_err(|e| Error::Internal(format!("Failed to fill background: {}", e)))?;
            
        // Draw a placeholder for now
        root.draw_text(
            "Pie Chart (PNG version)",
            &("sans-serif", 20),
            &BLACK,
            (width as i32 / 2, height as i32 / 2),
        ).map_err(|e| Error::Internal(format!("Failed to draw text: {}", e)))?;
        
        root.present().map_err(|e| Error::Internal(format!("Failed to render: {}", e)))?;
        
        Ok(buffer)
    }
}

// JSON Chart Renderer
pub struct JsonChartRenderer;

impl JsonChartRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl ChartRenderer for JsonChartRenderer {
    fn render_timeline(
        &self,
        data: &[serde_json::Value],
        _width: u32,
        _height: u32,
        _options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error> {
        // Create a JSON representation of the timeline data
        let mut timeline_data = Vec::new();
        
        for item in data {
            if let (Some(created_at), Some(entity_type), Some(value), Some(id)) = (
                item["created_at"].as_str(),
                item["entity_type"].as_str(),
                item["value"].as_str(),
                item["id"].as_str(),
            ) {
                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(created_at) {
                    timeline_data.push(serde_json::json!({
                        "id": id,
                        "date": date.to_rfc3339(),
                        "entity_type": entity_type,
                        "value": value,
                        "metadata": item.get("metadata").unwrap_or(&serde_json::json!({})),
                    }));
                }
            }
        }
        
        let result = serde_json::json!({
            "type": "timeline",
            "data": timeline_data,
        });
        
        serde_json::to_vec(&result)
            .map_err(|e| Error::Internal(format!("Failed to serialize chart data: {}", e)))
    }
    
    fn render_bar(
        &self,
        data: &[serde_json::Value],
        _width: u32,
        _height: u32,
        _options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error> {
        // Group data by entity_type
        let mut type_counts: HashMap<String, i32> = HashMap::new();
        
        for item in data {
            if let Some(entity_type) = item["entity_type"].as_str() {
                *type_counts.entry(entity_type.to_string()).or_insert(0) += 1;
            }
        }
        
        // Convert to array of objects
        let bar_data: Vec<serde_json::Value> = type_counts.iter()
            .map(|(key, value)| {
                serde_json::json!({
                    "category": key,
                    "value": value,
                })
            })
            .collect();
        
        let result = serde_json::json!({
            "type": "bar",
            "data": bar_data,
        });
        
        serde_json::to_vec(&result)
            .map_err(|e| Error::Internal(format!("Failed to serialize chart data: {}", e)))
    }
    
    fn render_pie(
        &self,
        data: &[serde_json::Value],
        _width: u32,
        _height: u32,
        _options: Option<&HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error> {
        // Group data by entity_type
        let mut type_counts: HashMap<String, i32> = HashMap::new();
        
        for item in data {
            if let Some(entity_type) = item["entity_type"].as_str() {
                *type_counts.entry(entity_type.to_string()).or_insert(0) += 1;
            }
        }
        
        // Convert to array of objects
        let pie_data: Vec<serde_json::Value> = type_counts.iter()
            .map(|(key, value)| {
                serde_json::json!({
                    "label": key,
                    "value": value,
                })
            })
            .collect();
        
        let result = serde_json::json!({
            "type": "pie",
            "data": pie_data,
        });
        
        serde_json::to_vec(&result)
            .map_err(|e| Error::Internal(format!("Failed to serialize chart data: {}", e)))
    }
}
