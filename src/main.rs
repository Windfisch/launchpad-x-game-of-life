use jack;

fn main() {
	let client = jack::Client::new("Game of Life", jack::ClientOptions::NO_START_SERVER).expect("Failed to connect to JACK").0;
	let in_port = client.register_port("in", jack::MidiIn).unwrap();
	let mut out_port = client.register_port("out", jack::MidiOut).unwrap();

	const TIME_STEP: u32 = 11025;
	let mut time = 0;
	let mut first = true;
	let mut field: [ [u16; 8] ; 8 ] = [ [0; 8] ; 8 ];

	// place a glider on the field
	field[3][3] = 1;
	field[4][4] = 1;
	field[2][5] = 1;
	field[3][5] = 1;
	field[4][5] = 1;

	let _async_client = client.activate_async((), jack::ClosureProcessHandler::new(move |_client: &jack::Client, scope: &jack::ProcessScope| -> jack::Control {
		time += scope.n_frames();

		let old_field = field;

		let mut writer = out_port.writer(scope);
		
		for foo in in_port.iter(scope) {
			if foo.bytes.len() == 3 && foo.bytes[0] == 0x90 && foo.bytes[2] != 0 {
				let id = foo.bytes[1];
				let x = (id as usize % 10) - 1;
				let y = (id as usize / 10) - 1;

				field[x][y] = if field[x][y] == 0 { 1 } else { 0 };
			}
		
			if first {
				// switch launchpad into programmer mode
				println!("switching to programmer mode");
				writer.write(&jack::RawMidi { time: 0, bytes: &[0xF0, 0x00, 0x20, 0x29, 0x02, 0x0C, 0x0E, 0x01, 0xF7] }).unwrap();
				first = false;
			}
		}

		const AGE_DIV: u16 = 1;

		let mut new_field = field;
		if time >= TIME_STEP {
			time -= TIME_STEP;

			for x1 in 0..8 {
				let x0 = (x1 + 7) % 8;
				let x2 = (x1 + 1) % 8;
				for y1 in 0..8 {
					let y0 = (y1 + 7) % 8;
					let y2 = (y1 + 1) % 8;

					let neighbors : Vec<_> = [(x0,y0), (x0,y1), (x0,y2), (x1,y0), (x1,y2), (x2,y0), (x2,y1), (x2,y2)].iter().filter_map(|(x,y)| if field[*x][*y] != 0 { Some(field[*x][*y]) } else { None } ).collect();
					let n_neighbors = neighbors.len() as u16;

					if field[x1][y1] != 0 && (n_neighbors == 2 || n_neighbors == 3) {
						new_field[x1][y1] += 1;
					}
					else if field[x1][y1] == 0 && n_neighbors == 3 {
						new_field[x1][y1] += 1;
					}
					else {
						new_field[x1][y1] = 0;
					}

					if new_field[x1][y1] >= 7*AGE_DIV { new_field[x1][y1] = 7*AGE_DIV }
				}
			}
			
		}
			


		for x1 in 0..8 {
			for y1 in 0..8 {
				if old_field[x1][y1] != new_field[x1][y1] {
					let color = if new_field[x1][y1] != 0 { (new_field[x1][y1] / AGE_DIV)*8+1 } else { 0 };
					writer.write(&jack::RawMidi { time: 0, bytes: &[0x90, (x1 as u8+1) + (y1 as u8+1)*10, color as u8] }).unwrap();
				}
			}
		}
		field = new_field;
		return jack::Control::Continue;
	})).expect("Failed to activate client");

	loop {}
}
