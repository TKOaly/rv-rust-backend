pub enum Actions {
    ItemCreated,
    ChangedDescription,
    ChangedWeight,
    ChangedProductGroup,
    BoughtBy,
    ChangedBuyprice,
    ChangedSellprice,
    ChangedCountOrInventoriedButNotBought,
    UserCreated,
    LoggedIn,
    LoggedOut,
    LoginFailed,
    FailedPasswordChange,
    AdminChangedPassword,
    SelfChangedPassword,
    WithdrewMoney,
    #[deprecated(note = "Use `DepositedMoneyCash` or `DepositedMoneyBankTransfer` instead")]
    DepositedMoney,
    ChangedRole,
    ChangedName,
    ChangedUnivident,
    #[deprecated(note = "Feature removed")]
    ChangedBuzzerlimit,
    #[deprecated(note = "Feature removed")]
    ChangedFgcolor,
    #[deprecated(note = "Feature removed")]
    ChangedBgcolor,
    BoxCreated,
    ChangedBoxItemCount,
    DepositedMoneyCash,
    DepositedMoneyBankTransfer,
    ProductReturned,
    ProductBuyIn,
}

impl From<Actions> for i32 {
    #[allow(deprecated)]
    fn from(action: Actions) -> Self {
        match action {
            Actions::ItemCreated => 1,
            Actions::ChangedDescription => 2,
            Actions::ChangedWeight => 3,
            Actions::ChangedProductGroup => 4,
            Actions::BoughtBy => 5,
            Actions::ChangedBuyprice => 6,
            Actions::ChangedSellprice => 7,
            Actions::ChangedCountOrInventoriedButNotBought => 8,
            Actions::UserCreated => 9,
            Actions::LoggedIn => 10,
            Actions::LoggedOut => 11,
            Actions::LoginFailed => 12,
            Actions::FailedPasswordChange => 13,
            Actions::AdminChangedPassword => 14,
            Actions::SelfChangedPassword => 15,
            Actions::WithdrewMoney => 16,
            Actions::DepositedMoney => 17,
            Actions::ChangedRole => 18,
            Actions::ChangedName => 19,
            Actions::ChangedUnivident => 20,
            Actions::ChangedBuzzerlimit => 21,
            Actions::ChangedFgcolor => 22,
            Actions::ChangedBgcolor => 23,
            Actions::BoxCreated => 24,
            Actions::ChangedBoxItemCount => 25,
            Actions::DepositedMoneyCash => 26,
            Actions::DepositedMoneyBankTransfer => 27,
            Actions::ProductReturned => 28,
            Actions::ProductBuyIn => 29,
        }
    }
}
