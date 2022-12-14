use crate::core::Indexer;
use anyhow::Error;
use libh3_sys::{degsToRads, edgeLengthKm, geoToH3, kRing, maxKringSize, GeoCoord, H3Index};

#[derive(Debug, Clone)]
pub(crate) struct H3Indexer {
    resolution: i32,
}

impl H3Indexer {
    pub(crate) fn new(resolution: i32) -> Result<Self, Error> {
        if resolution < 0 || resolution > 15 {
            return Err(Error::msg(format!("invalid resolution for h3 indexer: {}", resolution)));
        }
        Ok(Self { resolution })
    }
}

impl<'a> Indexer<'a, i64> for H3Indexer {
    fn index(&self, latitude: f64, longitude: f64) -> i64 {
        let coord = GeoCoord {
            lat: unsafe { degsToRads(latitude) },
            lon: unsafe { degsToRads(longitude) },
        };
        (unsafe { geoToH3(&coord as *const GeoCoord, self.resolution) }) as i64
    }

    fn neighbors(&self, index: i64, distance: f64) -> Vec<i64> {
        let k = ((distance / 1000.0 - unsafe { edgeLengthKm(self.resolution) })
            / unsafe { (edgeLengthKm(self.resolution)) * 2.0 })
        .ceil() as i32;
        let mut res = vec![0u64; unsafe { maxKringSize(k) } as usize];
        unsafe {
            kRing(index as u64, k, &mut res[0] as *mut H3Index);
        }
        res.into_iter().map(|v| v as i64).collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_index() {
        let indexer = H3Indexer::new(8).unwrap();
        let idx = indexer.index(36.657004, 117.0242607);
        println!("{:x}", idx);
        let neighbors = indexer.neighbors(idx, 0.5);
        for n in neighbors {
            println!("{:x}", n);
        }
    }
}
