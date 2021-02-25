mod utils;
mod processor;
mod mmu;
mod ic;
mod ram;
mod ppu;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
