use std::io::StdoutLock;
use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

// Depending on the type field, there an arbitrary number of
// keys avaialble in the message body. One way is to use generics,
// but we do not know the types before hand.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Body {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to_id: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Echo {echo : String},
    EchoOk { echo: String},
    InitOk {
        msg_id: usize,
        node_id: usize,
        node_ids: Vec<usize>,
    }
}

struct EchoNode {
    id: usize,
}

impl EchoNode {
    pub fn step(
        &mut self,
        input: Message,
        // for sending response back
        output: &mut serde_json::Serializer<StdoutLock>,
    ) -> anyhow::Result<()> {
        match input.body.payload {
            // do nothing when we get EchoOk
            Payload::EchoOk { .. } => {},
            // extract echo to be used as EchoOk reply message payload
            Payload::Echo {echo } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to_id: input.body.id,
                        payload: Payload::EchoOk {
                            echo
                        },
                    }
                };

                reply.serialize(output)
                    .context("serialize response to EchoOk type")?;
            }

        }


        self.id += 1;
        Ok(())
    }

}

fn main() -> anyhow::Result<()>{
    // The lock() method is called on the stdin handle. This method provides a
    // StdinLock which is a type that allows you to read from stdin safely.
    // Locking the standard input ensures that the input stream is exclusive
    // to the current thread. This means that while one part of your code is
    // reading from stdin, no other part of the program (or other threads) can
    // read from stdin simultaneously.
    let stdin = std::io::stdin().lock();
    // Deserialize an instance of type T from an I/O stream of JSON.
    // into_iter() takes ownership of the vector result of`from_reader`
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();


    let stdout = std::io::stdout().lock();
    let mut output = serde_json::Serializer::new(stdout);

    let mut state = EchoNode { id: 0 };


      for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be deserialized.")?;
          state.step(input, &mut output).context("Node step function failed!")?;
    }

    Ok(())
}
