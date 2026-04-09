use std::pin::Pin;
use std::task::{Context, Poll};

use eventsource_stream::Eventsource;
use futures_core::Stream;
use futures_util::StreamExt;
use pin_project_lite::pin_project;

use crate::error::{AiChatError, Result};
use crate::types::chat::ChatStreamChunk;

pin_project! {
    /// An async stream that yields [`ChatStreamChunk`] items parsed from an SSE
    /// connection. Use `StreamExt::next()` to consume chunks one by one.
    pub struct ChatStream {
        #[pin]
        inner: Pin<Box<dyn Stream<Item = Result<ChatStreamChunk>> + Send>>,
    }
}

impl ChatStream {
    /// Wrap a raw `reqwest::Response` whose body is an SSE byte stream.
    pub(crate) fn new(response: reqwest::Response) -> Self {
        let byte_stream = response.bytes_stream();
        let event_stream = byte_stream.eventsource();

        let mapped = event_stream.filter_map(|event_result| async move {
            match event_result {
                Ok(event) => {
                    let data = event.data.trim().to_string();
                    if data == "[DONE]" || data.is_empty() {
                        return None;
                    }
                    Some(serde_json::from_str::<ChatStreamChunk>(&data).map_err(|e| {
                        AiChatError::Stream(format!("failed to parse chunk: {e}: {data}"))
                    }))
                }
                Err(e) => Some(Err(AiChatError::Stream(e.to_string()))),
            }
        });

        ChatStream {
            inner: Box::pin(mapped),
        }
    }

    /// Consume the entire stream and concatenate all `delta.content` fragments
    /// into a single `String`.
    pub async fn collect_text(mut self) -> Result<String> {
        let mut buf = String::new();
        while let Some(chunk) = self.next().await {
            let chunk = chunk?;
            for choice in &chunk.choices {
                if let Some(content) = &choice.delta.content {
                    buf.push_str(content);
                }
            }
        }
        Ok(buf)
    }
}

impl Stream for ChatStream {
    type Item = Result<ChatStreamChunk>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.as_mut().poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}
