-- concourse.dhall

let List/map =
      https://prelude.dhall-lang.org/v11.1.0/List/map sha256:dd845ffb4568d40327f2a817eb42d1c6138b929ca758d50bc33112ef3c885680

let Target = {
  name : Text,
  runner_platform : Optional Text
}

let targets : List Target = [
    { name = "x86_64-unknown-linux-gnu", runner_platform = Some "linux" },
    { name = "x86_64-unknown-linux-musl", runner_platform = Some "linux" },
    { name = "x86_64-windows-msvc", runner_platform = Some "windows" }
]

let GetStep = {
	get: Text,
    trigger: Bool
}

let InParallelStep = {
	in_parallel: {
    	fail_fast: Bool
    }
}

let TaskStep = {
	task: Text
}

let Step = < GetStep | InParallelStep | TaskStep >

let Job = {
	name: Text,
    plan: List Step
}

in  List/map Target List Step (\(target : Target) -> {
	
}) targets
