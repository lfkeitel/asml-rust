use super::parser::program::Program;

pub fn link(program: &mut Program) -> Result<(), String> {
    for part in &mut program.parts {
        for label_val in part.link_map.iter() {
            let loc = label_val.0.to_owned() as usize;
            let label = label_val.1;

            if let Some(memloc) = program.labels.get(&label.label) {
                let newloc = memloc + label.offset as u16;

                part.bytes[loc] = (newloc >> 8) as u8;
                part.bytes[loc + 1] = newloc as u8;
            } else {
                return Err(format!("label {} is not defined", label.label));
            }
        }
    }

    Ok(())
}
