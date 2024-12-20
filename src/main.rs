use std::collections::HashMap;

use anyhow::bail;

#[derive(Debug)]
struct Group {
    head_part: String,
    name_parts: Vec<String>,
    inner_groups: Vec<Vec<Group>>,
}

impl Group {
    fn show_name(&self, fixity: Fixity) -> String {
        let mut result = String::new();
        if fixity.has_prefix() {
            result.push('_')
        };
        result.push_str(&self.head_part);

        for part in &self.name_parts {
            result.push('_');
            result.push_str(part)
        }
        if fixity.has_postfix() {
            result.push('_')
        };
        result
    }
}

fn get_groups(
    input: &mut &[String],
    name_parts: &Vec<Vec<String>>,
    end_group: Option<&str>,
) -> anyhow::Result<Vec<Group>> {
    println!("Grouping {input:?}");
    let mut groups = Vec::new();
    while let [token, rest @ ..] = input {
        if let Some(parts) = name_parts.iter().find(|v| v.first() == Some(token)) {
            *input = rest;
            let mut inner_groups = Vec::new();
            for part in &parts[1..] {
                let groups = get_groups(input, name_parts, Some(part.as_str()))?;
                let [token, rest @ ..] = input else {
                    bail!("EOF found, '{part}' expected");
                };
                if token != part {
                    bail!("Expected '{part}', found '{token}'");
                };
                inner_groups.push(groups);
                *input = rest;
            }
            groups.push(Group {
                head_part: parts[0].clone(),
                name_parts: parts[1..].into_iter().cloned().collect(),
                inner_groups,
            })
        } else {
            if end_group == Some(token.as_str()) {
                break;
            }
            bail!("Failed to find valid group beginning with '{token}'");
        }
    }
    Ok(groups)
}

#[derive(Clone, Copy)]
enum Fixity {
    Infix(u32, u32),
    Prefix(u32),
    Postfix(u32),
    Nonfix,
}

impl Fixity {
    fn has_prefix(&self) -> bool {
        match self {
            Fixity::Infix(_, _) | Fixity::Prefix(_) => true,
            Fixity::Postfix(_) | Fixity::Nonfix => false,
        }
    }
    fn has_postfix(&self) -> bool {
        match self {
            Fixity::Infix(_, _) | Fixity::Postfix(_) => true,
            Fixity::Prefix(_) | Fixity::Nonfix => false,
        }
    }
}

#[derive(Debug)]
struct Application(String, Vec<Application>);

impl std::fmt::Display for Application {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}", self.0)?;
        for v in &self.1 {
            write!(f, " {}", v)?;
        }
        write!(f, ")")
    }
}

trait MutRefSliceHelper<'a> {
    fn peek(&self) -> Option<&'a Group>;
    fn next(&mut self) -> Option<&'a Group>;
}

impl<'a> MutRefSliceHelper<'a> for &'a [Group] {
    fn peek(&self) -> Option<&'a Group> {
        self.first()
    }

    fn next(&mut self) -> Option<&'a Group> {
        let [head, rest @ ..] = self else { return None };
        *self = rest;
        Some(head)
    }
}

fn pratt_parser(input: &mut &[Group], levels: &HashMap<String, Fixity>, power: u32) -> Application {
    println!(
        "Processing group {:?}",
        input
            .iter()
            .map(|g| g.show_name(levels.get(&g.head_part).copied().unwrap_or(Fixity::Nonfix)))
            .collect::<Vec<_>>()
    );
    let Some(group) = input.next() else {
        panic!("Parser should never be called on empty group list.")
    };

    let mut lhs = match levels.get(&group.head_part) {
        Some(Fixity::Nonfix) => Application(
            group.show_name(Fixity::Nonfix),
            group
                .inner_groups
                .iter()
                .map(|group| {
                    let mut group_ref = group.as_slice();
                    let argument = pratt_parser(&mut group_ref, levels, 0);
                    assert_eq!(group_ref.len(), 0);
                    argument
                })
                .collect(),
        ),
        Some(Fixity::Prefix(next_level)) => {
            let mut arguments: Vec<Application> = group
                .inner_groups
                .iter()
                .map(|group| {
                    let group = group.as_slice();
                    let mut group_ref: &[Group] = &group;
                    let argument = pratt_parser(&mut group_ref, levels, 0);
                    assert_eq!(group_ref.len(), 0);
                    argument
                })
                .collect();
            arguments.push(pratt_parser(input, levels, *next_level));
            Application(group.show_name(Fixity::Prefix(*next_level)), arguments)
        }
        Some(_) => panic!(
            "Expected prefix or infix operator. Found {}",
            group.head_part
        ),
        None => panic!("Unknown start of group {}", group.head_part),
    };

    while let Some(group) = input.peek() {
        println!(
            "lhs: {lhs:?}, op: {}",
            group.show_name(
                levels
                    .get(&group.head_part)
                    .copied()
                    .unwrap_or(Fixity::Nonfix)
            )
        );
        println!(
            "rhs from: {:?}",
            input
                .iter()
                .map(|g| g.show_name(levels.get(&g.head_part).copied().unwrap_or(Fixity::Nonfix)))
                .collect::<Vec<_>>()
        );
        match levels.get(&group.head_part) {
            Some(Fixity::Postfix(binding_power)) => {
                if *binding_power < power {
                    break;
                }
                input.next();

                let mut arguments = vec![lhs];
                arguments.extend(group.inner_groups.iter().map(|group| {
                    let mut group_ref = group.as_slice();
                    let argument = pratt_parser(&mut group_ref, levels, 0);
                    assert_eq!(group_ref.len(), 0);
                    argument
                }));

                lhs = Application(group.show_name(Fixity::Postfix(*binding_power)), arguments);
            }
            Some(Fixity::Infix(lhs_bp, rhs_bp)) => {
                if *lhs_bp < power {
                    break;
                }

                input.next();

                let mut arguments = vec![lhs];
                arguments.extend(group.inner_groups.iter().map(|group| {
                    let mut group_ref = group.as_slice();
                    let argument = pratt_parser(&mut group_ref, levels, 0);
                    assert_eq!(group_ref.len(), 0);
                    argument
                }));

                arguments.push(pratt_parser(input, levels, *rhs_bp));

                lhs = Application(group.show_name(Fixity::Infix(*lhs_bp, *rhs_bp)), arguments);
            }
            Some(_) => break,
            None => panic!("Precedence for {} not found.", group.head_part),
        }
    }

    lhs
}

fn main() {
    let levels = [
        ("+".to_owned(), Fixity::Infix(1, 2)),
        ("-".to_owned(), Fixity::Infix(1, 2)),
        ("*".to_owned(), Fixity::Infix(3, 4)),
        ("/".to_owned(), Fixity::Infix(3, 4)),
        ("x".to_owned(), Fixity::Nonfix),
        ("(".to_owned(), Fixity::Nonfix),
    ]
    .into_iter()
    .collect();
    let input = "x + x + x * x + ( x + x ) * x";
    let tokens = input
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();
    let groups = get_groups(
        &mut tokens.as_slice(),
        &vec![
            vec!["+".to_owned()],
            vec!["-".to_owned()],
            vec!["*".to_owned()],
            vec!["/".to_owned()],
            vec!["(".to_owned(), ")".to_owned()],
            vec!["x".to_owned()],
        ],
        None,
    )
    .unwrap();

    println!("{}", pratt_parser(&mut groups.as_slice(), &levels, 0));
}
