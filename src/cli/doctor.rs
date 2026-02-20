use crate::use_cases;
pub fn run() {
    let use_case = use_cases::doctor::DoctorUseCase::new();
    use_case.execute();
}
