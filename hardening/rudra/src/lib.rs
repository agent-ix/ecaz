#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RudraItemPointer {
    pub block_number: u32,
    pub offset_number: u16,
}

impl RudraItemPointer {
    pub fn encode_into(self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.block_number.to_le_bytes());
        out.extend_from_slice(&self.offset_number.to_le_bytes());
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 6 {
            return Err("short item pointer");
        }
        let block_number = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let offset_number = u16::from_le_bytes([bytes[4], bytes[5]]);
        Ok(Self {
            block_number,
            offset_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RudraItemPointer;

    #[test]
    fn item_pointer_round_trips() {
        let pointer = RudraItemPointer {
            block_number: 42,
            offset_number: 7,
        };
        let mut encoded = Vec::new();
        pointer.encode_into(&mut encoded);
        assert_eq!(RudraItemPointer::decode(&encoded), Ok(pointer));
    }
}
