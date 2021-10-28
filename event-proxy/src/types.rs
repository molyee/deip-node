use substrate_subxt::{Runtime, ClientBuilder, system::System};

use super::frame::{DeipProposal, Deip, DeipDao};
#[cfg(feature = "octopus")]
use super::frame::{OctopusAppchain, OctopusLpos};


fn _register_types<T: Runtime>(c: ClientBuilder<T>) -> ClientBuilder<T>
    where
        T: System + DeipProposal + Deip + DeipDao,
{
    c
        // System:
        .register_type_size::<<T as System>::AccountId>("T::AccountId")
        // DeipProposal:
        .register_type_size::<<T as DeipProposal>::ProposalBatch>("ProposalBatch<T>")
        .register_type_size::<<T as DeipProposal>::ProposalId>("ProposalId")
        .register_type_size::<<T as DeipProposal>::ProposalState>("ProposalState")
        // Deip:               
        .register_type_size::<<T as Deip>::DomainId>("DomainId")
        .register_type_size::<<T as Deip>::ProjectId>("ProjectId")
        .register_type_size::<<T as Deip>::Project>("Project")
        .register_type_size::<<T as Deip>::ReviewId>("ReviewId")
        .register_type_size::<<T as Deip>::Review>("Review")
        .register_type_size::<<T as Deip>::NdaId>("NdaId")
        .register_type_size::<<T as Deip>::NdaAccessRequestId>("NdaAccessRequestId")
        .register_type_size::<<T as Deip>::ProjectContentId>("ProjectContentId")
        .register_type_size::<<T as Deip>::FundingModel>("FundingModel")
        .register_type_size::<<T as Deip>::InvestmentId>("InvestmentId")
        .register_type_size::<<T as Deip>::ContractAgreementId>("ContractAgreementId")
        .register_type_size::<<T as Deip>::ContractAgreementTerms>("ContractAgreementTerms")
        // DeipDao:
        .register_type_size::<<T as DeipDao>::Dao>("DaoOf<T>")
}

#[cfg(not(feature = "octopus"))]
pub fn register_types<T: Runtime>(c: ClientBuilder<T>) -> ClientBuilder<T>
    where
        T: System + DeipProposal + Deip + DeipDao,
{
    _register_types(c)
}

#[cfg(feature = "octopus")]
pub fn register_types<T: Runtime>(c: ClientBuilder<T>) -> ClientBuilder<T>
    where
        T: System + DeipProposal + Deip + DeipDao + OctopusAppchain + OctopusLpos,
{
    let c = _register_types(c);
    c
        // OctopusAppchain:
        .register_type_size::<<T as OctopusAppchain>::Balance>("BalanceOf<T>")
        .register_type_size::<<T as OctopusAppchain>::AssetBalance>("AssetBalanceOf<T>")
        .register_type_size::<<T as OctopusAppchain>::AssetId>("AssetIdOf<T>")
        // OctopusLpos:
        .register_type_size::<<T as OctopusAppchain>::AssetId>("EraIndex")
}
