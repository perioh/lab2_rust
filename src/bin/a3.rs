use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};

use agency::*;
use rand::{random, Rng};

fn main() {
    let elements_count = 10;
    let vacancies = (0..elements_count)
        .map(|_| generate_vacancy())
        .collect::<Vec<_>>();

    let now = Instant::now();
    let _avg_salary_concurrent = avg_salary_concurrent(&vacancies);
    let elapsed_avg_salary_concurrent = now.elapsed();

    let now = Instant::now();
    let _max_salary_concurrent = max_salary_concurrent(&vacancies);
    let elapsed_max_salary_concurrent = now.elapsed();

    let now = Instant::now();
    let _avg_salary_sequential = avg_salary_sequential(&vacancies);
    let elapsed_avg_salary_sequential = now.elapsed();

    let now = Instant::now();
    let _max_salary_sequential = max_salary_sequential(&vacancies);
    let elapsed_max_salary_sequential = now.elapsed();

    println!("\n\n\n\t\tConcurrent\tSequential");
    println!(
        "Max salary (ns)\t{}\t\t{}",
        elapsed_max_salary_concurrent.as_nanos(),
        elapsed_max_salary_sequential.as_nanos()
    );
    println!(
        "Avg salary (ns)\t{}\t\t{}",
        elapsed_avg_salary_concurrent.as_nanos(),
        elapsed_avg_salary_sequential.as_nanos()
    );
}

fn generate_vacancy() -> Vacancy {
    let mut rng = rand::thread_rng();

    Vacancy::new(
        "Company",
        "Specialization",
        "Conditions",
        &rng.gen_range(0..10000).to_string(),
        Some("IT".to_owned()),
        &random::<u16>().to_string(),
        "",
    )
    .expect("Should generate fine")
}

fn avg_salary_concurrent(arr: &[Vacancy]) -> usize {
    let sum_salaries = Arc::new(AtomicUsize::new(0));
    let mut all_spawns = vec![];
    for vacancy in arr {
        let salary = vacancy.salary();
        // let vacancy = vacancy.clone();
        let sum_salaries = sum_salaries.clone();

        let spawn = std::thread::spawn(move || sum_salaries.fetch_add(salary, Ordering::Relaxed));
        all_spawns.push(spawn);
        // std::thread::spawn(f)
    }
    all_spawns.into_iter().map(|spawn| spawn.join()).count();
    sum_salaries.load(Ordering::Relaxed) / arr.len()
}

fn max_salary_concurrent(arr: &[Vacancy]) -> usize {
    let max_salary = Arc::new(Mutex::new(0));
    let mut all_spawns = vec![];
    for vacancy in arr {
        let salary = vacancy.salary();
        let max_salary = max_salary.clone();

        let spawn = std::thread::spawn(move || {
            let mut max_salary = max_salary.lock().unwrap();
            if salary > *max_salary {
                *max_salary = salary;
            }
        });
        all_spawns.push(spawn);
    }
    all_spawns.into_iter().map(|spawn| spawn.join()).count();
    let max_salary = *max_salary.lock().unwrap();
    max_salary
}

fn avg_salary_sequential(arr: &[Vacancy]) -> usize {
    let mut sum_salaries = 0;
    for vacancy in arr {
        sum_salaries += vacancy.salary();
        // std::thread::spawn(f)
    }

    sum_salaries / arr.len()
}

fn max_salary_sequential(arr: &[Vacancy]) -> usize {
    let mut max_salary = 0;
    for vacancy in arr {
        if vacancy.salary() > max_salary {
            max_salary = vacancy.salary();
        }
        // std::thread::spawn(f)
    }

    max_salary
}

mod agency {
    use serde::Deserialize;
    use serde::Serialize;
    use std::fmt::Display;

    // #[derive(Debug, Clone, Serialize, Deserialize)]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Vacancy {
        company_name: String,
        specialization: String,
        conditions: String,
        sallary: usize,
        worker_requirements: WorkerRequirements,
    }

    impl Display for Vacancy {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}: {}({}) - {}$ {}",
                self.company_name,
                self.specialization,
                self.conditions,
                self.sallary,
                self.worker_requirements.education
            )
        }
    }

    pub trait VacancyTrait {
        fn company_name(&self) -> &str;
        fn specialization(&self) -> &str;
        fn education(&self) -> &Education;
        fn salary(&self) -> usize;
    }
    impl VacancyTrait for Vacancy {
        fn salary(&self) -> usize {
            self.sallary
        }
        fn company_name(&self) -> &str {
            &self.company_name
        }

        fn specialization(&self) -> &str {
            &self.specialization
        }

        fn education(&self) -> &Education {
            &self.worker_requirements.education
        }
    }

    #[derive(Debug)]
    pub enum VacancyCreationError {
        ParsingEducationString(String),
        ParsingSallary(String),
        ParsingExperience(String),
    }
    impl Vacancy {
        pub fn new(
            company_name: &str,
            specialization: &str,
            conditions: &str,
            sallary_string: &str,
            worker_specialization_name: Option<String>,
            work_exp_years_string: &str,
            education_string: &str,
        ) -> Result<Self, VacancyCreationError> {
            let work_exp_years: u16 = work_exp_years_string.parse().map_err(|_| {
                VacancyCreationError::ParsingExperience(work_exp_years_string.to_owned())
            })?;

            let worker_specialization =
                if let Some(specialization_name) = worker_specialization_name {
                    Some(WorkerSpecialization {
                        specialization_name,
                        work_exp_years,
                    })
                } else {
                    None
                };
            let sallary: usize = sallary_string
                .parse()
                .map_err(|_| VacancyCreationError::ParsingSallary(sallary_string.to_owned()))?;
            let education = match &*education_string.to_lowercase() {
                "" => Education::None,
                "school" => Education::School,
                "university" => Education::University,
                _ => {
                    return Err(VacancyCreationError::ParsingEducationString(
                        education_string.to_owned(),
                    ))
                }
            };
            Ok(Self {
                company_name: company_name.to_owned(),
                specialization: specialization.to_owned(),
                conditions: conditions.to_owned(),
                sallary,
                worker_requirements: WorkerRequirements {
                    specialization: worker_specialization,
                    education,
                },
            })
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct WorkerRequirements {
        specialization: Option<WorkerSpecialization>,
        education: Education,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct WorkerSpecialization {
        specialization_name: String,
        work_exp_years: u16,
    }
    #[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Serialize, Deserialize)]
    pub enum Education {
        None,
        School,
        University,
    }

    impl Display for Education {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use Education::*;
            let edu_str = match self {
                None => "",
                School => "school",
                University => "university",
            };
            write!(f, "{edu_str}")
        }
    }
}
