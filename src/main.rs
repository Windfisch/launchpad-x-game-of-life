use jack;

fn main() {
	let client = jack::Client::new("Game of Life", jack::ClientOptions::NO_START_SERVER).expect("Failed to connect to JACK").0;
	let in_port = client.register_port("in", jack::MidiIn).unwrap();
	let mut out_port = client.register_port("out", jack::MidiOut).unwrap();

	const TIME_STEP: u32 = 11025;
	let mut time = 0;
	let mut field: [ [bool; 8] ; 8 ] = [ [false; 8] ; 8 ];
	field[3][3] = true;
	field[4][4] = true;
	field[2][5] = true;
	field[3][5] = true;
	field[4][5] = true;
	let _async_client = client.activate_async((), jack::ClosureProcessHandler::new(move |_client: &jack::Client, scope: &jack::ProcessScope| -> jack::Control {
		time += scope.n_frames();

		let old_field = field;

		for foo in in_port.iter(scope) {
			if foo.bytes.len() == 3 && foo.bytes[0] == 0x90 && foo.bytes[2] != 0 {
				let id = foo.bytes[1];
				let x = (id as usize % 10) - 1;
				let y = (id as usize / 10) - 1;

				field[x][y] = !field[x][y];
			}
		}

		let mut new_field = field;
		if time >= TIME_STEP {
			time -= TIME_STEP;

			for x1 in 0..8 {
				let x0 = (x1 + 7) % 8;
				let x2 = (x1 + 1) % 8;
				for y1 in 0..8 {
					let y0 = (y1 + 7) % 8;
					let y2 = (y1 + 1) % 8;

					let neighbors = [(x0,y0), (x0,y1), (x0,y2), (x1,y0), (x1,y2), (x2,y0), (x2,y1), (x2,y2)].iter().filter(|(x,y)| field[*x][*y]).count();
					if field[x1][y1] && (neighbors == 2 || neighbors == 3) {
						new_field[x1][y1] = true;
					}
					else if !field[x1][y1] && neighbors == 3 {
						new_field[x1][y1] = true;
					}
					else {
						new_field[x1][y1] = false;
					}
				}
			}
			
		}
			
		let mut writer = out_port.writer(scope);

		for x1 in 0..8 {
			for y1 in 0..8 {
				if old_field[x1][y1] != new_field[x1][y1] {
					writer.write(&jack::RawMidi { time: 0, bytes: &[0x90, (x1 as u8+1) + (y1 as u8+1)*10, if new_field[x1][y1] { 64 } else { 0 }] }).unwrap();
				}
			}
		}
		field = new_field;
		return jack::Control::Continue;
	})).expect("Failed to activate client");

	loop {}
}
