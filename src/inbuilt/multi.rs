pub fn multi(input_args: Vec<String>, qued_commands: &mut Vec<String>) -> String {
    qued_commands.push(input_args.join(" "));

    return String::from("OK");
}