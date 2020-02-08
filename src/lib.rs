use reqwest::{blocking::Client, Error as HttpError};
use serde::Deserialize;
use std::{borrow::Borrow, fmt};

#[cfg(test)]
mod test;

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: for<'t> Deserialize<'t>"))]
struct CFResult<T: for<'t> Deserialize<'t>> {
    result: Option<T>,
    comment: Option<String>,
}

impl<T: for<'t> Deserialize<'t>> From<CFResult<T>> for Result<T> {
    fn from(c: CFResult<T>) -> Self {
        match c.result {
            Some(v) => Ok(v),
            None => Err(Error::Codeforces(
                c.comment.unwrap_or("Unknown error".to_owned()),
            )),
        }
    }
}

/// The error returned.
#[derive(Debug)]
pub enum Error {
    /// Occurred from within reqwest.
    Http(HttpError),
    /// Decoding error,
    Decode(serde_json::Error),
    /// Sent back from codeforces.
    Codeforces(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(ref e) => write!(f, "HTTP: {}", e),
            Error::Decode(ref e) => write!(f, "Decode: {}", e),
            Error::Codeforces(ref s) => write!(f, "Codeforces: {}", s),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(ref e) => Some(e),
            Error::Decode(ref e) => Some(e),
            Error::Codeforces(_) => None,
        }
    }
}

impl From<HttpError> for Error {
    fn from(e: HttpError) -> Self {
        Error::Http(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Decode(e)
    }
}

/// The result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A codeforces user.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub handle: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub country: Option<String>,
    pub organization: Option<String>,
    pub city: Option<String>,
    pub contribution: i64,
    pub rank: Option<String>,
    pub max_rank: Option<String>,
    pub rating: Option<i64>,
    pub max_rating: Option<i64>,
    pub last_online_time_seconds: u64,
    pub registration_time_seconds: u64,
    pub friend_of_count: u64,
    pub avatar: String,
    pub title_photo: String,
}

impl User {
    /// URL to the profile of the user.
    pub fn profile_url(&self) -> String {
        format!("https://codeforces.com/profile/{}", self.handle)
    }

    /// The color of their username.
    pub fn color(&self) -> u64 {
        match self.rating {
            None => 0x000000,
            Some(rating) => {
                if rating < 1200 {
                    0x808080
                } else if rating < 1400 {
                    0x008000
                } else if rating < 1600 {
                    0x03a89e
                } else if rating < 1900 {
                    0x0000ff
                } else if rating < 2100 {
                    0xaa00aa
                } else if rating < 2300 {
                    0xbbbb00
                } else if rating < 2400 {
                    0xff8c00
                } else {
                    0xff0000
                }
            }
        }
    }
}

/// An user's rating change.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RatingChange {
    pub contest_id: u64,
    pub contest_name: String,
    pub handle: String,
    pub rank: u64,
    pub rating_update_time_seconds: u64,
    pub old_rating: i64,
    pub new_rating: i64,
}

/// The scoring type of a contest.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ContestType {
    CF,
    IOI,
    ICPC,
}

impl fmt::Display for ContestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContestType::CF => write!(f, "Codeforces"),
            ContestType::IOI => write!(f, "IOI-based"),
            ContestType::ICPC => write!(f, "ACM ICPC-based"),
        }
    }
}

/// The current phase of the contest.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContestPhase {
    Before,
    Coding,
    PendingSystemTest,
    SystemTest,
    Finished,
}

impl fmt::Display for ContestPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ContestPhase::*;
        write!(
            f,
            "{}",
            match self {
                Before => "Contest hasn't started",
                Coding => "Contest is currently running",
                PendingSystemTest => "Pending system test",
                SystemTest => "System test running",
                Finished => "Finished",
            }
        )
    }
}

/// A single contest.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contest {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub contest_type: ContestType,
    pub phase: ContestPhase,
    pub frozen: bool,
    pub duration_seconds: u64,
    pub start_time_seconds: Option<u64>,
    pub relative_time_seconds: Option<u64>,
    pub prepared_by: Option<String>,
    pub website_url: Option<String>,
    pub description: Option<String>,
    pub difficulty: Option<u8>,
    pub kind: Option<String>,
    pub icpc_region: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub season: Option<String>,
}

impl Contest {
    /// URL to the contest.
    pub fn url(&self) -> String {
        format!("https://codeforces.com/contests/{}", self.id)
    }
}

/// The type of a problem.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProblemType {
    Programming,
    Question,
}

impl fmt::Display for ProblemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProblemType::Programming => write!(f, "Programming"),
            ProblemType::Question => write!(f, "Question"),
        }
    }
}

/// Represents a problem.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    pub contest_id: Option<u64>,
    pub problemset_name: Option<String>,
    pub index: String,
    pub name: String,
    #[serde(rename = "type")]
    pub problem_type: ProblemType,
    pub points: Option<f64>,
    pub rating: Option<u64>,
    pub tags: Vec<String>,
}

/// A team member.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub handle: String,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParticipantType {
    Contestant,
    Practice,
    Virtual,
    Manager,
    OutOfCompetition,
}

impl fmt::Display for ParticipantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParticipantType::*;
        write!(
            f,
            "{}",
            match self {
                Contestant => "Contestant",
                Practice => "Practice",
                Virtual => "Virtual",
                Manager => "Manager",
                OutOfCompetition => "OutOfCompetition",
            }
        )
    }
}

/// A group of participants.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Party {
    pub contest_id: Option<u64>,
    pub members: Vec<TeamMember>,
    pub participant_type: ParticipantType,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub ghost: bool,
    pub room: Option<u64>,
    pub start_time_seconds: Option<u64>,
}

/// Either the result is Preliminary or Final
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProblemResultType {
    Preliminary,
    Final,
}

impl fmt::Display for ProblemResultType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ProblemResultType::Preliminary => "Preliminary",
                ProblemResultType::Final => "Final",
            }
        )
    }
}

/// One party's result on a particular problem.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProblemResult {
    pub points: f64,
    pub penalty: Option<u64>,
    pub rejected_attempt_count: u64,
    #[serde(rename = "type")]
    pub result_type: ProblemResultType,
    pub best_submission_time_seconds: Option<u64>,
}

/// A row in the scoreboard.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RanklistRow {
    pub party: Party,
    pub rank: u64,
    pub points: f64,
    pub penalty: u64,
    pub successful_hack_count: u64,
    pub unsuccessful_hack_count: u64,
    pub problem_results: Vec<ProblemResult>,
    pub last_submission_time_seconds: Option<u64>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Verdict {
    Failed,
    Ok,
    Partial,
    CompilationError,
    RuntimeError,
    WrongAnswer,
    PresentationError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    IdlenessLimitExceeded,
    SecurityViolated,
    Crashed,
    InputPreparationCrashed,
    Challenged,
    Skipped,
    Testing,
    Rejected,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Verdict::*;
        write!(
            f,
            "{}",
            match self {
                Failed => "Failed",
                Ok => "Ok",
                Partial => "Partial",
                CompilationError => "Compilation Error",
                RuntimeError => "Runtime Error",
                WrongAnswer => "Wrong Answer",
                PresentationError => "Presentation Error",
                TimeLimitExceeded => "Time Limit Exceeded",
                MemoryLimitExceeded => "Memory Limit Exceeded",
                IdlenessLimitExceeded => "Idleness Limit Exceeded",
                SecurityViolated => "Security Violated",
                Crashed => "Crashed",
                InputPreparationCrashed => "Input Preparation Crashed",
                Challenged => "Challenged",
                Skipped => "Skipped",
                Testing => "Testing",
                Rejected => "Rejected",
            }
        )
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubmissionTestSet {
    Samples,
    Pretests,
    Tests,
    Challenges,
    #[serde(rename = "TESTS1")]
    TestSet1,
    #[serde(rename = "TESTS2")]
    TestSet2,
    #[serde(rename = "TESTS3")]
    TestSet3,
    #[serde(rename = "TESTS4")]
    TestSet4,
    #[serde(rename = "TESTS5")]
    TestSet5,
    #[serde(rename = "TESTS6")]
    TestSet6,
    #[serde(rename = "TESTS7")]
    TestSet7,
    #[serde(rename = "TESTS8")]
    TestSet8,
    #[serde(rename = "TESTS9")]
    TestSet9,
    #[serde(rename = "TESTS10")]
    TestSet10,
}

impl fmt::Display for SubmissionTestSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use SubmissionTestSet::*;
        write!(
            f,
            "{}",
            match self {
                Samples => "Samples",
                Pretests => "Pretests",
                Tests => "Tests",
                Challenges => "Challenges",
                TestSet1 => "Test Set 1",
                TestSet2 => "Test Set 2",
                TestSet3 => "Test Set 3",
                TestSet4 => "Test Set 4",
                TestSet5 => "Test Set 5",
                TestSet6 => "Test Set 6",
                TestSet7 => "Test Set 7",
                TestSet8 => "Test Set 8",
                TestSet9 => "Test Set 9",
                TestSet10 => "Test Set 10",
            }
        )
    }
}

/// Represents a submission.
///
/// https://codeforces.com/apiHelp/objects#Submission
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub id: u64,
    pub contest_id: Option<u64>,
    pub creation_time_seconds: u64,
    pub relative_time_seconds: Option<u64>,
    pub problem: Problem,
    pub author: Party,
    pub programming_language: String,
    pub verdict: Option<Verdict>,
    #[serde(rename = "testset")]
    pub test_set: SubmissionTestSet,
    pub passed_test_count: u64,
    pub time_consumed_millis: u64,
    pub memory_consumed_bytes: u64,
}

/// API methods described on Codeforces API page.
impl User {
    /// Returns information about one or several users.
    ///
    /// https://codeforces.com/apiHelp/methods#user.info
    pub fn info<T>(client: &Client, handles: &[T]) -> Result<Vec<User>>
    where
        T: Borrow<str>,
    {
        let users: CFResult<_> = client
            .get("https://codeforces.com/api/user.info")
            .query(&[("handles", handles.join(";"))])
            .send()?
            .json()?;
        users.into()
    }

    /// Returns the list users who have participated in at least one rated contest.
    ///
    /// The return list of Users are sorted by decreasing order of rating.
    ///
    /// https://codeforces.com/apiHelp/methods#user.ratedList
    pub fn rated_list(client: &Client, active_only: bool) -> Result<Vec<User>> {
        let users = client
            .get("https://codeforces.com/api/user.ratedList")
            .query(&[("activeOnly", active_only)])
            .send()?;
        let users: CFResult<_> = serde_json::from_reader(users)?;
        users.into()
    }

    /// Returns rating history of the specified user.
    ///
    /// https://codeforces.com/apiHelp/methods#user.rating
    pub fn rating(client: &Client, handle: &str) -> Result<Vec<RatingChange>> {
        let changes: CFResult<_> = client
            .get("https://codeforces.com/api/user.rating")
            .query(&[("handle", handle)])
            .send()?
            .json()?;
        changes.into()
    }

    /// Returns submissions of specified user.
    ///
    /// https://codeforces.com/apiHelp/methods#user.status
    pub fn status(client: &Client, handle: &str, from: u64, count: u64) -> Result<Vec<Submission>> {
        let submissions: CFResult<_> = client
            .get("https://codeforces.com/api/user.status")
            .query(&[("handle", handle)])
            .query(&[("from", from.max(1)), ("count", count.min(1))])
            .send()?
            .json()?;
        submissions.into()
    }
}

/// Build a contest ranking request.
#[derive(Debug, Default)]
pub struct ContestRankingsBuilder {
    from: Option<u64>,
    count: Option<u64>,
    handles: Option<Vec<String>>,
    room: Option<u64>,
    allow_unofficial: bool,
}

impl ContestRankingsBuilder {
    /// Put a limit on the number of records returned.
    pub fn limit(&mut self, from: u64, count: u64) -> &mut Self {
        self.from = Some(from);
        self.count = Some(count);
        self
    }

    /// Set a list of handles.
    pub fn handles(&mut self, handles: Vec<String>) -> &mut Self {
        self.handles = Some(handles);
        self
    }

    /// Set a specific room.
    pub fn room(&mut self, room: u64) -> &mut Self {
        self.room = Some(room);
        self
    }

    /// Allow unofficial contestants.
    pub fn allow_unofficial(&mut self, value: bool) -> &mut Self {
        self.allow_unofficial = value;
        self
    }
}

/// Consumes self and return a query list.
impl From<ContestRankingsBuilder> for Vec<(&'static str, String)> {
    fn from(c: ContestRankingsBuilder) -> Self {
        vec![
            Some(("allowOfficial", c.allow_unofficial.to_string())),
            c.from.map(|v| ("from", v.to_string())),
            c.count.map(|v| ("count", v.to_string())),
            c.handles.map(|v| ("handles", v.join(";"))),
            c.room.map(|v| ("room", v.to_string())),
        ]
        .into_iter()
        .filter_map(|v| v)
        .collect()
    }
}

/// API methods described on Codeforces API page.
impl Contest {
    /// Gets a list of all contests.
    pub fn list(client: &Client, with_gym: bool) -> Result<Vec<Contest>> {
        let v: CFResult<_> = client
            .get("https://codeforces.com/api/contest.list")
            .query(&[("gym", with_gym)])
            .send()?
            .json()?;
        v.into()
    }

    /// Gets the standings of a contest.
    ///
    /// https://codeforces.com/apiHelp/methods#contest.standings
    pub fn standings(
        client: &Client,
        contest_id: u64,
        opts: impl FnOnce(&mut ContestRankingsBuilder) -> &mut ContestRankingsBuilder,
    ) -> Result<(Contest, Vec<Problem>, Vec<RanklistRow>)> {
        #[derive(Deserialize)]
        struct Middle {
            contest: Contest,
            problems: Vec<Problem>,
            rows: Vec<RanklistRow>,
        }

        let mut b = ContestRankingsBuilder::default();
        opts(&mut b);

        let v: CFResult<Middle> = client
            .get("https://codeforces.com/api/contest.standings")
            .query(&[("contestId", contest_id)])
            .query(&Vec::<(&'static str, String)>::from(b))
            .send()?
            .json()?;
        let v: Middle = Result::<_>::from(v)?;

        Ok((v.contest, v.problems, v.rows))
    }
}

/// APIs provided as methods.
impl User {
    /// Gets a list of rating changes of the current user.
    pub fn rating_changes(&self, client: &Client) -> Result<Vec<RatingChange>> {
        Self::rating(client, &self.handle)
    }

    /// Gets a list of most recent submissions.
    pub fn submissions(&self, client: &Client, from: u64, count: u64) -> Result<Vec<Submission>> {
        Self::status(client, &self.handle, from, count)
    }
}

/// APIs provided as methods.
impl Contest {
    /// Get the standings of the current contest.
    pub fn get_standings(
        &self,
        client: &Client,
        opts: impl FnOnce(&mut ContestRankingsBuilder) -> &mut ContestRankingsBuilder,
    ) -> Result<(Vec<Problem>, Vec<RanklistRow>)> {
        let (_, problems, rows) = Self::standings(client, self.id, opts)?;
        Ok((problems, rows))
    }
}
