 // Add methods on commands
 // Used for writing assertions

 // Run programs


extern crate rexpect;

use rexpect::errors::*;
use rexpect::session::PtyReplSession;
use rexpect::spawn;

fn test_p2p_ctl_signals() -> Result<()> {
    let mut p2p = PtyReplSession {
        // for `echo_on` you need to figure that out by trial and error.
        // For bash and python repl it is false
        echo_on: false,

        // used for `wait_for_prompt()`
        prompt: "".to_string(),
        pty_session: spawn("p2p", Some(2000))?,
        // command which is sent when the instance of this struct is dropped
        // in the below example this is not needed, but if you don't explicitly
        // exit a REPL then rexpect tries to send a SIGTERM and depending on the repl
        // this does not end the repl and would end up in an error
        quit_command: Some("quit".to_string()),
    };
    p2p.wait_for_prompt()?;
    p2p.send_line("ls p")?;
    p2p.wait_for_prompt()?;
    p2p.send_control('c')?;
    p2p.exp_eof()?;
    Ok(())
}

fn setup_p2p_single(){

}

fn setup_p2p_triple(){

}

fn teardown_p2p_triple(){

}

fn test_local_cassette_write() {
    //
}

#[test]
fn test_p2p_session() {
    assert!(test_p2p_ctl_signals().is_ok())
}

// #[test]
// fn test_vcr_cassettes() {
//     let pr = setup_p2p_triple();
//     assert!(test_local_cassette_write(rgtt).is_ok())
// }
