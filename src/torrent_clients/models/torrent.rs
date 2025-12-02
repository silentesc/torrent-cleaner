use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Torrent {
    hash: String,
    name: String,
    total_size: i64,
    content_path: String,
    ratio: f32,
    state: String,
    tracker: String,
    category: String,
    tags: String,
    added_on: i64,
    completion_on: i64,
    seeding_time: i64,
}

impl Torrent {
    pub fn new(
        hash: String,
        name: String,
        total_size: i64,
        content_path: String,
        ratio: f32,
        state: String,
        tracker: String,
        category: String,
        tags: String,
        added_on: i64,
        completion_on: i64,
        seeding_time: i64,
    ) -> Self {
        Torrent {
            hash,
            name,
            total_size,
            content_path,
            ratio,
            state,
            tracker,
            category,
            tags,
            added_on,
            completion_on,
            seeding_time,
        }
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn total_size(&self) -> &i64 {
        &self.total_size
    }
    pub fn content_path(&self) -> &str {
        &self.content_path
    }
    pub fn ratio(&self) -> &f32 {
        &self.ratio
    }
    pub fn state(&self) -> &str {
        &self.state
    }
    pub fn tracker(&self) -> &str {
        &self.tracker
    }
    pub fn category(&self) -> &str {
        &self.category
    }
    pub fn tags(&self) -> &str {
        &self.tags
    }
    pub fn added_on(&self) -> &i64 {
        &self.added_on
    }
    pub fn completion_on(&self) -> &i64 {
        &self.completion_on
    }
    pub fn seeding_time(&self) -> &i64 {
        &self.seeding_time
    }

    pub fn println(&self) {
        println!("Torrent {}", self.hash);
        println!("  name:          {}", self.name);
        println!("  total_size:    {}", self.total_size);
        println!("  content_path:  {}", self.content_path);
        println!("  ratio:         {}", self.ratio);
        println!("  state:         {}", self.state);
        println!("  tracker:       {}", self.tracker);
        println!("  category:      {}", self.category);
        println!("  tags:          {}", self.tags);
        println!("  added_on:      {}", self.added_on);
        println!("  completion_on: {}", self.completion_on);
        println!("  seeding_time:  {}", self.seeding_time);
    }
}
