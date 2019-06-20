use super::error::ForthError;
use super::RustForth;

impl RustForth {
    pub fn internal_mul(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x * y;

        self.push_stack(result);

        println!("Multiplied {} by {} resulting in {}", x, y, result);

        Ok(())
    }

    pub fn internal_div(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x / y;

        self.push_stack(result);

        println!("Divided {} by {} resulting in {}", x, y, result);

        Ok(())
    }

    pub fn internal_add(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x + y;

        self.push_stack(result);

        println!("Added {} to {} resulting in {}", x, y, result);

        Ok(())
    }

    pub fn internal_sub(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x - y;

        self.push_stack(result);

        println!("Subtracted {} by {} resulting in {}", x, y, result);

        Ok(())
    }
    pub fn internal_dup(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;

        self.push_stack(x);
        self.push_stack(x);

        println!("Duplicated {} ", x);

        Ok(())
    }
}

impl RustForth {
    pub fn push_stack(&mut self, n: i64) {
        println!("Pushed {} on stack", n);
        self.number_stack.push(n);
    }

    pub fn pop_stack(&mut self) -> Result<i64, ForthError> {
        println!("Popped stack");
        match self.number_stack.pop() {
            Some(x) => Ok(x),
            None => Err(ForthError::PopOfEmptyStack),
        }
    }
}