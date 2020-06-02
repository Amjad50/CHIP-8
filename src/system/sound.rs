use rodio::{source::SineWave, Sink};

pub struct Sound {
    sink: Sink,
}

impl Sound {
    pub fn new(freq: u32) -> Self {
        let device = rodio::default_output_device().unwrap();
        let sink = Sink::new(&device);

        // Add a dummy source of the sake of the example.
        let source = SineWave::new(freq);
        sink.append(source);
        sink.pause(); // Start without playing.

        Sound { sink: sink }
    }

    pub fn play(&mut self) {
        self.sink.play();
    }

    pub fn stop(&mut self) {
        self.sink.pause();
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
}
