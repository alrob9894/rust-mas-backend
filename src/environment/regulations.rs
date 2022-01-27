pub struct Regulations{
    pub mask_mandate: bool,
    pub social_distancing: bool,
}

impl Regulations {
    pub fn new() -> Regulations {
        Regulations {
            mask_mandate: false,
            social_distancing: false,
        }
    }
}
