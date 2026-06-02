macro_rules! add_bench {
    () => {
        bench!(async_channel);
        bench!(flume);
        bench!(futures_mpsc);
        bench!(tachyonix);
        bench!(thingbuf);
        bench!(postage_mpsc);
        bench!(tokio_mpsc);
        bench!(chenal);
        bench!(chenal_mpmc);
        bench!(chenal_mpmc_racy);
        bench!(chenal2);
        bench!(chenal2_mpmc);
        bench!(chenal2_mpmc_racy);
        bench!(chenal_vyukov);
        bench!(crossfire);
        bench!(kanal);
    };
}

pub(crate) use add_bench;
