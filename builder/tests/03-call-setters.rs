// Generate methods on the builder for setting a value of each of the struct
// fields.
//
//     impl CommandBuilder {
//         fn executable(&mut self, executable: String) -> &mut Self {
//             self.executable = Some(executable);
//             self
//         }
//         fn args(&mut self, args: Vec<String>) -> &mut Self {
//              self.args = Some(args);
//              self
//          }
//         fn env(&mut self, env: Vec<String>) -> &mut Self {
//              self.args = Some(args);
//              self
//          }
//         fn current_dir(&mut self, current_dir: String) -> &mut Self {
//              self.args = Some(current_dir);
//              self
//          }
//     }

use derive_builder::Builder;

#[derive(Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
}

fn main() {
    let mut builder = Command::builder();
    builder.executable("cargo".to_owned());
    builder.args(vec!["build".to_owned(), "--release".to_owned()]);
    builder.env(vec![]);
    builder.current_dir("..".to_owned());
}
