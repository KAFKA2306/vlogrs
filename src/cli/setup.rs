use crate::infrastructure;
use crate::use_cases;
pub fn run() {
    let check_env = infrastructure::fs_utils::LocalEnvironment;
    let use_case = use_cases::SetupUseCase::new(Box::new(check_env));
    use_case.execute();
}
