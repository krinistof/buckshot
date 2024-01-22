#[derive(Debug)]
struct Player {
    name: String,
    lives: u8,
    cuffed: bool,
    inventory: Vec<Item>,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            name: "player".into(),
            lives: 2,
            cuffed: false,
            inventory: vec![],
        }
    }
}

impl Player {
    fn try_saw(&mut self, gun: &mut Shotgun) -> Option<()> {
        use crate::Item::Saw;
        let mut inventory = &mut self.inventory;

        let saw_index = inventory.iter().position(|i| *i == Saw)?;
        inventory.remove(saw_index);

        gun.sawed = true;
        Some(())
    }

    fn try_magnifier(&mut self, gun: &Shotgun) -> Option<Bullet> {
        use crate::Item::Magnifier;
        let mut inventory = &mut self.inventory;

        let index = inventory.iter().position(|i| *i == Magnifier)?;
        inventory.remove(index);

        Some(*gun.magazine.last()?)
    }

    fn try_beer(&mut self, gun: &mut Shotgun) -> Option<Bullet> {
        use crate::Item::Beer;
        let mut inventory = &mut self.inventory;

        let index = inventory.iter().position(|i| *i == Beer)?;
        inventory.remove(index);

        gun.magazine.pop()
    }

    fn try_cuff(&mut self, other: &mut Player) -> Option<()> {
        use crate::Item::Handcuffs;
        let mut inventory = &mut self.inventory;

        let index = inventory.iter().position(|i| *i == Handcuffs)?;

        other.cuffed = true;
        Some(())
    }

    fn try_cigarette(&mut self) -> Option<()> {
        use crate::Item::Cigarette;
        let mut inventory = &mut self.inventory;

        let index = inventory.iter().position(|i| *i == Cigarette)?;
        inventory.remove(index);

        self.lives += 1;
        Some(())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Bullet {
    Blank,
    Live,
}

#[derive(Debug)]
struct Shotgun {
    magazine: Vec<Bullet>,
    sawed: bool,
}

impl Default for Shotgun {
    fn default() -> Self {
        Shotgun {
            magazine: vec![],
            sawed: false,
        }
    }
}

impl Shotgun {
    pub fn pull_on(&mut self, player: &mut Player) {
        let bullet = self.magazine.pop();
        match bullet {
            Some(ref shell) => {
                use Bullet::*;
                match shell {
                    Live => {
                        self.damage(player);
                    }
                    Blank => {}
                }
            }
            None => {
                panic!();
            }
        }
        self.sawed = false;
    }

    fn damage(&self, player: &mut Player) {
        player.lives -= if self.sawed { 2 } else { 1 }
    }

    pub fn randomize_bullets(&mut self) {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        self.magazine.shuffle(&mut rng);
    }
}

#[derive(Debug, PartialEq)]
enum Item {
    Cuff,
    Cigarette,
    Saw,
    Beer,
    Magnifier,
    Handcuffs,
}

fn help() {
    println!("commands: me, you");
}

fn status(players: &Vec<Player>) {
    for player in players {
        let name = &player.name;
        let lives = &player.lives;
        println!("{name}: {lives}");
    }
}
fn main() {
    use Bullet::*;

    let mut players = vec![
        Player {
            name: "a".into(),
            ..Default::default()
        },
        Player {
            name: "b".into(),
            ..Default::default()
        },
    ];

    let mut gun = Shotgun {
        magazine: vec![Live, Live, Blank, Blank],
        ..Default::default()
    };
    gun.randomize_bullets();

    let mut player_index = 0;
    help();
    while players.iter().all(|p| p.lives > 0) && gun.magazine.len() > 0 {
        status(&players);

        let len = players.len();
        let mut player = &mut players[player_index];

        let name = &player.name;
        println!("current: {name}");
        loop {
            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).unwrap();
            match choice.trim() {
                "me" => gun.pull_on(player),
                "you" => gun.pull_on(player),
                &_ => {
                    dbg!(choice);
                    help();
                }
            }
            break;
        }

        player_index += 1;
        player_index %= len;
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use Bullet::*;
    use Item::*;

    // Source for tests: https://youtu.be/yj6iTCE6OBE

    #[test]
    fn actual_first_level() {
        let mut player = Player {
            name: "name".into(),
            ..Default::default()
        };
        let mut dealer = Player {
            name: "dealer".into(),
            ..Default::default()
        };

        let mut shotgun = Shotgun {
            magazine: vec![Blank, Live, Blank],
            ..Default::default()
        };

        assert_eq!(shotgun.magazine.len(), 3);
        shotgun.pull_on(&mut player);
        assert_eq!(shotgun.magazine.len(), 2);
        assert_eq!(player.lives, 2);

        shotgun.pull_on(&mut dealer);
        assert_eq!(shotgun.magazine.len(), 1);
        assert_eq!(dealer.lives, 1);

        shotgun.magazine = vec![Live, Blank, Live, Blank, Live];
        assert_eq!(shotgun.magazine.len(), 5);

        shotgun.pull_on(&mut dealer);
        assert_eq!(shotgun.magazine.len(), 4);
        assert_eq!(dealer.lives, 0);
    }

    #[test]
    fn actual_second_level() {
        let mut player = Player {
            name: "name".into(),
            lives: 4,
            inventory: vec![Saw, Handcuffs],
            ..Default::default()
        };
        let mut dealer = Player {
            name: "dealer".into(),
            lives: 4,
            inventory: vec![Saw, Saw],
            ..Default::default()
        };

        let mut gun = Shotgun {
            magazine: vec![Live, Blank],
            ..Default::default()
        };

        assert!(player.inventory.contains(&Saw));
        player.try_saw(&mut gun);
        assert!(!player.inventory.contains(&Saw));

        gun.sawed = true;
        assert!(gun.sawed);
        assert_eq!(gun.magazine.len(), 2);

        gun.pull_on(&mut player);
        assert_eq!(gun.magazine.len(), 1);
        assert_eq!(player.lives, 4);

        gun.pull_on(&mut dealer);
        assert_eq!(dealer.lives, 3);

        player.inventory.extend(vec![Magnifier, Saw]);
        dealer.inventory.extend(vec![Magnifier, Beer]);

        gun.magazine = vec![Live, Blank, Blank, Live];

        assert!(player.inventory.contains(&Magnifier));
        let next = player.try_magnifier(&gun).unwrap();
        assert!(!player.inventory.contains(&Magnifier));

        assert_eq!(next, Live);

        player.try_saw(&mut gun);

        gun.pull_on(&mut dealer);
        assert_eq!(dealer.lives, 1);

        let next = dealer.try_magnifier(&gun).unwrap();
        assert_eq!(next, Blank);

        let next = dealer.try_beer(&mut gun).unwrap();
        assert_eq!(next, Blank);

        assert_eq!(dealer.lives, 1);
        gun.pull_on(&mut dealer);
        assert_eq!(dealer.lives, 1);

        dealer.try_saw(&mut gun);

        assert_eq!(player.lives, 4);
        gun.pull_on(&mut player);
        assert_eq!(player.lives, 2);

        player.inventory.extend(vec![Cigarette, Magnifier]);
        dealer.inventory.extend(vec![Cigarette, Handcuffs]);

        gun.magazine = vec![Live, Blank, Live, Blank, Live];

        player.try_cigarette();
        assert_eq!(player.lives, 3);

        gun.pull_on(&mut dealer);
        assert_eq!(dealer.lives, 0);
    }

    #[test]
    fn actual_third_level() {
        let mut player = Player {
            name: "name".into(),
            lives: 6,
            inventory: vec![Cigarette, Cigarette, Magnifier, Magnifier],
            ..Default::default()
        };
        let mut dealer = Player {
            name: "dealer".into(),
            lives: 6,
            inventory: vec![Cigarette, Cigarette, Magnifier, Beer],
            ..Default::default()
        };

        let mut gun = Shotgun {
            magazine: vec![Live, Blank, Blank],
            ..Default::default()
        };

        let next = player.try_magnifier(&gun).unwrap();
        assert_eq!(next, Blank);

        gun.pull_on(&mut player);
        assert_eq!(player.lives, 6);

        gun.pull_on(&mut dealer);
        assert_eq!(dealer.lives, 6);

        gun.pull_on(&mut player);
        assert_eq!(player.lives, 5);

        player.inventory.extend(vec![Handcuffs, Saw, Saw, Beer]);
        dealer
            .inventory
            .extend(vec![Handcuffs, Handcuffs, Magnifier, Saw]);

        gun.magazine = vec![Live, Live, Live, Blank, Blank, Live, Blank, Blank];

        player.try_cigarette();
        let next = player.try_magnifier(&gun).unwrap();
        assert_eq!(next, Blank);

        player.try_saw(&mut gun);

        assert!(!dealer.cuffed);
        player.try_cuff(&mut dealer);
        assert!(dealer.cuffed);

        gun.pull_on(&mut dealer);
        assert_eq!(dealer.lives, 6);
    }
}
