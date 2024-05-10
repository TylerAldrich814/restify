use std::fmt::Debug;
use std::ops::{ControlFlow, FromResidual, Residual, Try};
use std::process::{ExitCode, Termination};
#[derive(Debug)]
pub enum Parsed<F, N> {
	Found(F),
	NotFound(N),
}

pub use Parsed::*;
impl<F, N> FromResidual for Parsed<F, N> {
	fn from_residual(residual: <Self as Try>::Residual) -> Self {
		match residual {
			NotFound(err) => NotFound(err),
			_ => unreachable!(),
		}
	}
}

impl<F, N> Try for Parsed<F, N> {
	type Output = F;
	type Residual = Parsed<F, N>;
	
	fn from_output(output: Self::Output) -> Self {
		Found(output)
	}
	
	fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
		match self {
			Found(val)
			=> ControlFlow::Continue(val),
			NotFound(err)
			=> ControlFlow::Break(NotFound(err)),
		}
	}
}

impl<F, N> Termination for Parsed<F, N>
	where
		F: Debug,
		N: Debug,
{
	fn report(self) -> ExitCode {
		match self {
			Found(_) => {
				println!("{:?}", self);
				ExitCode::SUCCESS
			},
			NotFound(err) => {
				eprintln!("{:?}", err);
				ExitCode::FAILURE
			}
		}
	}
}
