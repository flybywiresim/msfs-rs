use msfs::{nvg, MSFSEvent};

#[msfs::gauge(name=DEMO)]
async fn demo(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    let nvg = gauge.create_nanovg().unwrap();

    while let Some(event) = gauge.next_event().await {
        match event {
            MSFSEvent::PreDraw(d) => {
                nvg.draw_frame(d.width(), d.height(), |f| {
                    // draw black background
                    f.draw_path(nvg::Style::new().fill(nvg::Color::from_rgb(0, 0, 0)), |p| {
                        p.rect(0.0, 0.0, d.width() as f32, d.height() as f32);

                        Ok(())
                    })?;

                    // draw square
                    f.draw_path(
                        nvg::Style::new().fill(nvg::Color::from_rgb(255, 255, 255)),
                        |p| {
                            p.rect(20.0, 20.0, 40.0, 40.0);

                            Ok(())
                        },
                    )?;

                    Ok(())
                });
            }
            _ => {}
        }
    }

    Ok(())
}
