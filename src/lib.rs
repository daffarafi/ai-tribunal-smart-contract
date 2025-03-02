use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet},
    env, log, near, near_bindgen, AccountId, BorshStorageKey, CryptoHash, NearSchema, NearToken,
    PanicOnDefault, Promise, PromiseOrValue,
};

#[near]
pub struct Debate {
    topic: String,
    creator: AccountId,
    created_at: u64,
    figure_1_name: String,
    figure_1_image_url: String,
    figure_2_name: String,
    figure_2_image_url: String,
    debate_dialogue: Vec<(String, String, String)>,
}

#[near]
pub struct Vote {
    debate_id: u64,
    voter: AccountId,
    voted_at: u64,
    choice: u8,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    owner_id: AccountId,
    debates: UnorderedMap<u64, Debate>,
    next_debate_id: u64,
    votes: UnorderedMap<u64, Vote>,
    next_vote_id: u64,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            debates: UnorderedMap::new(StorageKey::Debates),
            next_debate_id: 1,
            votes: UnorderedMap::new(StorageKey::Votes),
            next_vote_id: 1,
        }
    }
}

#[near]
impl Contract {
    pub fn create_debate(
        &mut self,
        topic: String,
        figure_1_name: String,
        figure_1_image_url: String,
        figure_2_name: String,
        figure_2_image_url: String,
        debate_dialogue: Vec<(String, String, String)>,
    ) -> u64 {
        let debate_id = self.next_debate_id;
        self.next_debate_id += 1;

        let creator = env::signer_account_id();

        let debate = Debate {
            topic,
            creator,
            created_at: env::block_timestamp_ms(),
            figure_1_name,
            figure_1_image_url,
            figure_2_name,
            figure_2_image_url,
            debate_dialogue,
        };

        self.debates.insert(&debate_id, &debate);
        log!("Debate {} created successfully", debate_id);

        debate_id
    }

    pub fn vote_debate(&mut self, debate_id: u64, choice: u8) -> u64 {
        let voter = env::signer_account_id();

        if self.debates.get(&debate_id).is_none() {
            env::panic_str("Debate not found!");
        }

        if choice != 1 && choice != 2 {
            env::panic_str("Invalid choice! Choose 1 or 2.");
        }

        for (_, existing_vote) in self.votes.iter() {
            if existing_vote.debate_id == debate_id && existing_vote.voter == voter {
                env::panic_str("You have already voted in this debate!");
            }
        }

        let vote_id = self.next_vote_id;
        self.next_vote_id += 1;

        let vote = Vote {
            debate_id,
            voter: voter.clone(),
            voted_at: env::block_timestamp_ms(),
            choice,
        };

        self.votes.insert(&vote_id, &vote);
        log!(
            "{} voted for choice {} in debate {}",
            voter,
            choice,
            debate_id
        );

        vote_id
    }

    pub fn get_debates(
        &self,
    ) -> Vec<(
        u64,
        String,
        AccountId,
        u64,
        String,
        String,
        String,
        String,
        u64,
        u64,
    )> {
        let mut debate_list = Vec::new();

        for (debate_id, debate) in self.debates.iter() {
            let mut figure_1_votes = 0;
            let mut figure_2_votes = 0;

            for (_, vote) in self.votes.iter() {
                if vote.debate_id == debate_id {
                    if vote.choice == 1 {
                        figure_1_votes += 1;
                    } else if vote.choice == 2 {
                        figure_2_votes += 1;
                    }
                }
            }

            debate_list.push((
                debate_id,
                debate.topic.clone(),
                debate.creator.clone(),
                debate.created_at.clone(),
                debate.figure_1_name.clone(),
                debate.figure_1_image_url.clone(),
                debate.figure_2_name.clone(),
                debate.figure_2_image_url.clone(),
                figure_1_votes,
                figure_2_votes,
            ));
        }

        debate_list
    }

    pub fn get_detail_debate(
        &self,
        debate_id: u64,
    ) -> Option<(
        u64,
        String,
        AccountId,
        u64,
        String,
        String,
        String,
        String,
        Vec<(String, String, String)>,
        u64,
        u64,
    )> {
        self.debates.get(&debate_id).map(|debate| {
            let mut figure_1_votes = 0;
            let mut figure_2_votes = 0;

            for (_, vote) in self.votes.iter() {
                if vote.debate_id == debate_id {
                    if vote.choice == 1 {
                        figure_1_votes += 1;
                    } else if vote.choice == 2 {
                        figure_2_votes += 1;
                    }
                }
            }

            (
                debate_id,
                debate.topic.clone(),
                debate.creator.clone(),
                debate.created_at.clone(),
                debate.figure_1_name.clone(),
                debate.figure_1_image_url.clone(),
                debate.figure_2_name.clone(),
                debate.figure_2_image_url.clone(),
                debate.debate_dialogue.clone(),
                figure_1_votes,
                figure_2_votes,
            )
        })
    }

    pub fn get_user_vote_in_debate(&self, debate_id: u64) -> Option<(u8, u64)> {
        let voter = env::signer_account_id();

        for (_, vote) in self.votes.iter() {
            if vote.debate_id == debate_id && vote.voter == voter {
                return Some((vote.choice, vote.voted_at));
            }
        }

        None // Jika user belum vote, kembalikan None
    }
}
