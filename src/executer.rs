use async_stream::stream;
use std::str::FromStr;

use futures::Stream;
use reqwest::{Method, Request, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Step {
    url: Url,
    method: String,
}

impl Step {
    pub fn new(url: Url, method: &str) -> Self {
        Step {
            url,
            method: method.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StepResult {
    status: u16,
}

pub struct Executer {
    client: reqwest::Client,
}

impl Executer {
    pub fn new(client: reqwest::Client) -> Self {
        Executer { client }
    }

    pub fn execute_steps(&self, steps: Vec<Step>) -> impl Stream<Item = StepResult> + Send + '_ {
        stream! {
            for step in steps {
                let result = Self::execute_step(self, step).await;
                yield result;
            }
        }
    }

    async fn execute_step(&self, step: Step) -> StepResult {
        let req = Request::new(
            Method::from_str(step.method.clone().as_str()).unwrap(),
            step.url.clone(),
        );
        let resp = self.client.execute(req).await.unwrap();

        StepResult {
            status: resp.status().as_u16(),
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use httptest::{matchers::*, responders::*, Expectation, Server};
    use reqwest::Url;

    use crate::executer::Executer;

    #[tokio::test]
    async fn execute_step_get() {
        let server = Server::run();

        let subject = Executer::new(reqwest::Client::new());

        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .respond_with(status_code(200)),
        );

        let actual = subject
            .execute_step(crate::executer::Step::new(
                Url::parse(&server.url("/foo").to_owned().to_string()).unwrap(),
                "GET",
            ))
            .await;

        assert_eq!(actual.status, 200);
    }

    #[tokio::test]
    async fn execute_steps_get() {
        let server = Server::run();

        let subject = Executer::new(reqwest::Client::new());

        server.expect(
            Expectation::matching(request::method_path("GET", "/hoge"))
                .respond_with(status_code(200)),
        );
        server.expect(
            Expectation::matching(request::method_path("GET", "/fuga"))
                .respond_with(status_code(200)),
        );

        let steps = vec![
            crate::executer::Step::new(
                Url::parse(&server.url("/hoge").to_string()).unwrap(),
                "GET",
            ),
            crate::executer::Step::new(
                Url::parse(&server.url("/fuga").to_string()).unwrap(),
                "GET",
            ),
        ];

        let results = subject.execute_steps(steps);
        futures::pin_mut!(results);
        while let Some(result) = results.next().await {
            assert_eq!(result.status, 200);
        }
    }
}
