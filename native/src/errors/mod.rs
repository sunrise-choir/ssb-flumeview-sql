// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    errors {
        ArgumentError {}
        ArgumentTypeError{}
        ArgumentToBufferError{}
        CreateBufferError{}
        SecretKeyError{}
        NotARecipient{}
        NapiError{}
    }
}
