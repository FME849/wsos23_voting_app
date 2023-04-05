use anchor_lang::prelude::*;

declare_id!("DAExTRo6cEotQAgtREjCrS6R2tL54VfgBWsP7xozMiht");

#[program]
pub mod wsos23_voting_app {
    use super::*;

    pub fn create_election(ctx: Context<CreateElection>, election_id: String) -> Result<()> {
        let election = &mut ctx.accounts.election_data;
        let signer = &mut ctx.accounts.signer;

        election.id = election_id;
        election.candidates = 0;
        election.stage = ElectionStage::Application;
        election.initiator = signer.key();

        Ok(())
    }

    pub fn apply(ctx: Context<Apply>) -> Result<()> {
        let election = &mut ctx.accounts.election_data;
        let signer = &mut ctx.accounts.signer;
        let candidate_data = &mut ctx.accounts.candidate_data;

        require!(election.stage == ElectionStage::Application, ElectionError::ApplicationIsClosed);

        election.candidates += 1;
        candidate_data.id = election.candidates;
        candidate_data.pubkey = signer.key();
        candidate_data.votes = 0;

        Ok(())
    }

    pub fn change_stage(ctx: Context<ChangeStage>, new_stage: ElectionStage) -> Result<()> {
        let election = &mut ctx.accounts.election_data;
        let signer = &mut ctx.accounts.signer;

        require!(election.initiator == signer.key(), ElectionError::WrongInitiator);
        require!(election.stage != ElectionStage::Closed, ElectionError::VotingEnded);

        match new_stage {
            ElectionStage::Voting => {
                return election.close_application();
            },
            ElectionStage::Closed => {
                return election.close_voting();
            },
            ElectionStage::Application => {
                return Err(ElectionError::WrongInitiator.into());
            }
        }
    }

    pub fn vote(ctx: Context<Vote>) -> Result<()> {
        let election = &mut ctx.accounts.election_data;
        let candidate_data = &mut ctx.accounts.candidate_data;
        let my_vote = &mut ctx.accounts.my_vote;

        require!(election.stage == ElectionStage::Voting, ElectionError::NotVotingStage);

        candidate_data.votes += 1;
        my_vote.pubkey = candidate_data.pubkey;

        election.record_vote(candidate_data.id, candidate_data.votes);
        
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(election_id: String)]
pub struct CreateElection<'info> {
    #[account(init, payer=signer, space=5000, seeds=[&(election_id).as_bytes().as_ref(), signer.key().as_ref()], bump)]
    pub election_data: Account<'info, ElectionData>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[account]
pub struct ElectionData {
    pub id: String,
    pub candidates: u64,
    pub stage: ElectionStage,
    pub initiator: Pubkey,
    pub winners_id: u64,
    pub winners_votes: u64,
}

impl ElectionData {
    pub fn close_application(&mut self) -> Result<()> {
        require!(self.stage == ElectionStage::Application, ElectionError::ApplicationIsClosed);

        if self.candidates == 1 {
                self.winners_id;
                self.stage = ElectionStage::Closed;
        } else {
            self.stage = ElectionStage::Voting;
        }

        Ok(())
    }

    pub fn close_voting(&mut self) -> Result<()> {
        require!(self.stage == ElectionStage::Voting, ElectionError::NotVotingStage);
        self.stage = ElectionStage::Closed;
        Ok(())
    }

    pub fn record_vote(&mut self, id: u64, votes: u64) {
        if self.winners_id != id {
            if self.winners_votes >= votes {
                return
            } 
            self.winners_id = id;
            self.winners_votes = votes;
        } else {
            self.winners_votes += 1;
        }
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, PartialEq, Eq, Clone)]
pub enum ElectionStage {
    Application,
    Voting,
    Closed
}

#[derive(Accounts)]
pub struct Apply<'info> {
    #[account(init, payer=signer, space=5000, seeds=[b"candidate", signer.key().as_ref(), election_data.key().as_ref()], bump)]
    pub candidate_data: Account<'info, CandidateData>,
    #[account(mut)]
    pub election_data: Account<'info, ElectionData>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Register<'info> {
    #[account(init, payer=signer, space=5000, seeds=[election_data.key().as_ref()], bump)]
    pub candidate_data: Account<'info, CandidateData>,
    pub election_data: Account<'info, ElectionData>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct CandidateData {
    pub id: u64,
    pub pubkey: Pubkey,
    pub votes: u64,
}

#[derive(Accounts)]
pub struct ChangeStage <'info>{
    #[account(mut)]
    pub election_data: Account<'info, ElectionData>,
    #[account(mut, address=election_data.initiator @ ElectionError::WrongInitiator)]
    pub signer: Signer<'info>
}

#[derive(Accounts)]
pub struct Vote <'info>{
    #[account(init, payer=signer, space=5000, seeds=[b"vote", signer.key().as_ref(), election_data.key().as_ref()], bump)]
    pub my_vote: Account<'info, MyVote>,
    #[account(mut)]
    pub candidate_data: Account<'info, CandidateData>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub election_data: Account<'info, ElectionData>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct MyVote {
    pub id: u64,
    pub pubkey: Pubkey,
}

#[error_code]
pub enum ElectionError {
    #[msg("The application is closed")]
    ApplicationIsClosed,

    #[msg("The publicKey does not match")]
    WrongPublicKey,

    #[msg("You are not Initiator")]
    WrongInitiator,

    #[msg("Not at votingStage")]
    NotVotingStage,

    #[msg("Voting stage has ended")]
    VotingEnded,
}