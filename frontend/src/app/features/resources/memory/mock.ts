export const memoryResourcesTreeMapMock: any = {
  "value": 2451584,
  "cacheValue": 2189352,
  "frames": [
    {
      "name": {
        "offset": "0001297a",
        "executable": "libpthread.so.0",
        "functionName": "read",
        "functionCategory": "systemLib"
      },
      "value": 1379956,
      "cacheValue": 1372864,
      "frames": [
        {
          "name": {
            "offset": "022fe583",
            "executable": "light-node",
            "functionName": "<std::process::ChildStderr as std::io::Read>::read::h2b39f2a53104c282",
            "functionCategory": "nodeRust"
          },
          "value": 1379932,
          "cacheValue": 1372840,
          "frames": [
            {
              "name": {
                "offset": "01a32f8b",
                "executable": "light-node",
                "functionName": "std::io::default_read_exact::h4c495042f7de6f39",
                "functionCategory": "nodeRust"
              },
              "value": 1379900,
              "cacheValue": 1372840,
              "frames": [
                {
                  "name": {
                    "offset": "01a13e56",
                    "executable": "light-node",
                    "functionName": "storage::commit_log::CommitLog::read::h102b5e3cf8eebcea",
                    "functionCategory": "nodeRust"
                  },
                  "value": 1379900,
                  "cacheValue": 1372840,
                  "frames": [
                    {
                      "name": {
                        "offset": "01a148a1",
                        "executable": "light-node",
                        "functionName": "<storage::commit_log::CommitLogs as storage::commit_log::CommitLogWithSchema<S>>::get::h7ac06f5552653339",
                        "functionCategory": "nodeRust"
                      },
                      "value": 1379900,
                      "cacheValue": 1372840,
                      "frames": [
                        {
                          "name": {
                            "offset": "01a04f58",
                            "executable": "light-node",
                            "functionName": "storage::block_storage::BlockStorage::get_block_by_level::h2e505710ea60804e",
                            "functionCategory": "nodeRust"
                          },
                          "value": 1280896,
                          "cacheValue": 1279052,
                          "frames": [
                            {
                              "name": {
                                "offset": "01a0513e",
                                "executable": "light-node",
                                "functionName": "storage::block_storage::BlockStorage::get_block_hash_by_level::hc9232aa5934009ec",
                                "functionCategory": "nodeRust"
                              },
                              "value": 1280896,
                              "cacheValue": 1279052,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "0131089b",
                                    "executable": "light-node",
                                    "functionName": "shell_automaton::service::storage_service::StorageServiceDefault::run_worker::h32bbe7ed768e960e",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 1280896,
                                  "cacheValue": 1279052,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "0138de7d",
                                        "executable": "light-node",
                                        "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h47962be7f657fa3a",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 1280896,
                                      "cacheValue": 1279052,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "00ed0494",
                                            "executable": "light-node",
                                            "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h8a52fef8fbba4f4f",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 1280896,
                                          "cacheValue": 1279052,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "023150e3",
                                                "executable": "light-node",
                                                "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                "functionCategory": "nodeRust"
                                              },
                                              "value": 1280896,
                                              "cacheValue": 1279052,
                                              "frames": [
                                                {
                                                  "name": {
                                                    "offset": "00101603",
                                                    "executable": "libc.so.6",
                                                    "functionName": "__clone",
                                                    "functionCategory": "systemLib"
                                                  },
                                                  "value": 1280896,
                                                  "cacheValue": 1279052,
                                                  "frames": []
                                                }
                                              ]
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        },
                        {
                          "name": {
                            "offset": "004ae5c1",
                            "executable": "light-node",
                            "functionName": "<monitoring::monitor::Monitor as tezedge_actor_system::actor::Actor>::pre_start::he2cb47a79643fce7",
                            "functionCategory": "nodeRust"
                          },
                          "value": 65316,
                          "cacheValue": 60132,
                          "frames": [
                            {
                              "name": {
                                "offset": "00336532",
                                "executable": "light-node",
                                "functionName": "tezedge_actor_system::kernel::mailbox::process_sys_msgs::h13d0167a5ccd7f83",
                                "functionCategory": "nodeRust"
                              },
                              "value": 65316,
                              "cacheValue": 60132,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "00334ad2",
                                    "executable": "light-node",
                                    "functionName": "tezedge_actor_system::kernel::mailbox::run_mailbox::h48afaaab5b77ca95",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 65316,
                                  "cacheValue": 60132,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "002e1c5d",
                                        "executable": "light-node",
                                        "functionName": "tokio::loom::std::unsafe_cell::UnsafeCell<T>::with_mut::h1c13baa2a325d027",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 65316,
                                      "cacheValue": 60132,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "002b794f",
                                            "executable": "light-node",
                                            "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::hab9de3645eda2676",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 65316,
                                          "cacheValue": 60132,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "02297acf",
                                                "executable": "light-node",
                                                "functionName": "std::thread::local::LocalKey<T>::with::hc00dafd9e6767220",
                                                "functionCategory": "nodeRust"
                                              },
                                              "value": 65316,
                                              "cacheValue": 60132,
                                              "frames": [
                                                {
                                                  "name": {
                                                    "offset": "022b06b1",
                                                    "executable": "light-node",
                                                    "functionName": "tokio::runtime::thread_pool::worker::Context::run_task::hf312f91a8ef327f4",
                                                    "functionCategory": "nodeRust"
                                                  },
                                                  "value": 65316,
                                                  "cacheValue": 60132,
                                                  "frames": [
                                                    {
                                                      "name": {
                                                        "offset": "022af4aa",
                                                        "executable": "light-node",
                                                        "functionName": "tokio::runtime::thread_pool::worker::Context::run::h6378a7a70d4c2172",
                                                        "functionCategory": "nodeRust"
                                                      },
                                                      "value": 65316,
                                                      "cacheValue": 60132,
                                                      "frames": [
                                                        {
                                                          "name": {
                                                            "offset": "0229f145",
                                                            "executable": "light-node",
                                                            "functionName": "tokio::macros::scoped_tls::ScopedKey<T>::set::h414b9ab1e9afa14d",
                                                            "functionCategory": "nodeRust"
                                                          },
                                                          "value": 65316,
                                                          "cacheValue": 60132,
                                                          "frames": [
                                                            {
                                                              "name": {
                                                                "offset": "022aef6b",
                                                                "executable": "light-node",
                                                                "functionName": "tokio::runtime::thread_pool::worker::run::habb95f20da6cdfc3",
                                                                "functionCategory": "nodeRust"
                                                              },
                                                              "value": 65316,
                                                              "cacheValue": 60132,
                                                              "frames": [
                                                                {
                                                                  "name": {
                                                                    "offset": "022b311e",
                                                                    "executable": "light-node",
                                                                    "functionName": "tokio::loom::std::unsafe_cell::UnsafeCell<T>::with_mut::hc8140dffc630e76d",
                                                                    "functionCategory": "nodeRust"
                                                                  },
                                                                  "value": 65316,
                                                                  "cacheValue": 60132,
                                                                  "frames": [
                                                                    {
                                                                      "name": {
                                                                        "offset": "022a84b7",
                                                                        "executable": "light-node",
                                                                        "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h3d1631b3e1d4a63f",
                                                                        "functionCategory": "nodeRust"
                                                                      },
                                                                      "value": 65316,
                                                                      "cacheValue": 60132,
                                                                      "frames": [
                                                                        {
                                                                          "name": {
                                                                            "offset": "022a228c",
                                                                            "executable": "light-node",
                                                                            "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 65316,
                                                                          "cacheValue": 60132,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "022992fa",
                                                                                "executable": "light-node",
                                                                                "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 65316,
                                                                              "cacheValue": 60132,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "02299941",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 65316,
                                                                                  "cacheValue": 60132,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "023150e3",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 65316,
                                                                                      "cacheValue": 60132,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "00101603",
                                                                                            "executable": "libc.so.6",
                                                                                            "functionName": "__clone",
                                                                                            "functionCategory": "systemLib"
                                                                                          },
                                                                                          "value": 65316,
                                                                                          "cacheValue": 60132,
                                                                                          "frames": []
                                                                                        }
                                                                                      ]
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        }
                                                                      ]
                                                                    }
                                                                  ]
                                                                }
                                                              ]
                                                            }
                                                          ]
                                                        }
                                                      ]
                                                    }
                                                  ]
                                                }
                                              ]
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        },
                        {
                          "name": {
                            "offset": "012fdc05",
                            "executable": "light-node",
                            "functionName": "<storage::block_storage::BlockStorage as storage::block_storage::BlockStorageReader>::get::hdc0901e4df6e9826",
                            "functionCategory": "nodeRust"
                          },
                          "value": 32828,
                          "cacheValue": 32828,
                          "frames": [
                            {
                              "name": {
                                "offset": "0131021c",
                                "executable": "light-node",
                                "functionName": "shell_automaton::service::storage_service::StorageServiceDefault::run_worker::h32bbe7ed768e960e",
                                "functionCategory": "nodeRust"
                              },
                              "value": 32828,
                              "cacheValue": 32828,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "0138de7d",
                                    "executable": "light-node",
                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h47962be7f657fa3a",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 32828,
                                  "cacheValue": 32828,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "00ed0494",
                                        "executable": "light-node",
                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h8a52fef8fbba4f4f",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 32828,
                                      "cacheValue": 32828,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "023150e3",
                                            "executable": "light-node",
                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 32828,
                                          "cacheValue": 32828,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "00101603",
                                                "executable": "libc.so.6",
                                                "functionName": "__clone",
                                                "functionCategory": "systemLib"
                                              },
                                              "value": 32828,
                                              "cacheValue": 32828,
                                              "frames": []
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        },
                        {
                          "name": {
                            "offset": "0081a2f6",
                            "executable": "light-node",
                            "functionName": "rpc::services::rewards_services::collect_cycle_rewards::{{closure}}::hd9442aec9cbaee04",
                            "functionCategory": "nodeRust"
                          },
                          "value": 860,
                          "cacheValue": 828,
                          "frames": [
                            {
                              "name": {
                                "offset": "007dddf9",
                                "executable": "light-node",
                                "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::had5bb7208317cd00",
                                "functionCategory": "nodeRust"
                              },
                              "value": 860,
                              "cacheValue": 828,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "007dfdc2",
                                    "executable": "light-node",
                                    "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::hb01acf8ed8c9d702",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 860,
                                  "cacheValue": 828,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "007a9fbd",
                                        "executable": "light-node",
                                        "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h18206e35d187d7d1",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 860,
                                      "cacheValue": 828,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "005ecbaa",
                                            "executable": "light-node",
                                            "functionName": "hyper::proto::h1::dispatch::Dispatcher<D,Bs,I,T>::poll_loop::h40e1d345256f842a",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 860,
                                          "cacheValue": 828,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "005ea661",
                                                "executable": "light-node",
                                                "functionName": "hyper::proto::h1::dispatch::Dispatcher<D,Bs,I,T>::poll_catch::h0ae925c7a324338f",
                                                "functionCategory": "nodeRust"
                                              },
                                              "value": 860,
                                              "cacheValue": 828,
                                              "frames": [
                                                {
                                                  "name": {
                                                    "offset": "00962bca",
                                                    "executable": "light-node",
                                                    "functionName": "<hyper::server::conn::upgrades::UpgradeableConnection<I,S,E> as core::future::future::Future>::poll::h29798eb7efc140e0",
                                                    "functionCategory": "nodeRust"
                                                  },
                                                  "value": 860,
                                                  "cacheValue": 828,
                                                  "frames": [
                                                    {
                                                      "name": {
                                                        "offset": "00758cd1",
                                                        "executable": "light-node",
                                                        "functionName": "<hyper::server::conn::spawn_all::NewSvcTask<I,N,S,E,W> as core::future::future::Future>::poll::h75511f118733fcbd",
                                                        "functionCategory": "nodeRust"
                                                      },
                                                      "value": 860,
                                                      "cacheValue": 828,
                                                      "frames": [
                                                        {
                                                          "name": {
                                                            "offset": "0095a65a",
                                                            "executable": "light-node",
                                                            "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h37d09748e26f8785",
                                                            "functionCategory": "nodeRust"
                                                          },
                                                          "value": 860,
                                                          "cacheValue": 828,
                                                          "frames": [
                                                            {
                                                              "name": {
                                                                "offset": "006d1fc9",
                                                                "executable": "light-node",
                                                                "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::hc61ea0732ea65cc2",
                                                                "functionCategory": "nodeRust"
                                                              },
                                                              "value": 860,
                                                              "cacheValue": 828,
                                                              "frames": [
                                                                {
                                                                  "name": {
                                                                    "offset": "022979b5",
                                                                    "executable": "light-node",
                                                                    "functionName": "std::thread::local::LocalKey<T>::with::hc00dafd9e6767220",
                                                                    "functionCategory": "nodeRust"
                                                                  },
                                                                  "value": 860,
                                                                  "cacheValue": 828,
                                                                  "frames": [
                                                                    {
                                                                      "name": {
                                                                        "offset": "022b06b1",
                                                                        "executable": "light-node",
                                                                        "functionName": "tokio::runtime::thread_pool::worker::Context::run_task::hf312f91a8ef327f4",
                                                                        "functionCategory": "nodeRust"
                                                                      },
                                                                      "value": 860,
                                                                      "cacheValue": 828,
                                                                      "frames": [
                                                                        {
                                                                          "name": {
                                                                            "offset": "022af4aa",
                                                                            "executable": "light-node",
                                                                            "functionName": "tokio::runtime::thread_pool::worker::Context::run::h6378a7a70d4c2172",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 860,
                                                                          "cacheValue": 828,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "0229f145",
                                                                                "executable": "light-node",
                                                                                "functionName": "tokio::macros::scoped_tls::ScopedKey<T>::set::h414b9ab1e9afa14d",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 860,
                                                                              "cacheValue": 828,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "022aef6b",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "tokio::runtime::thread_pool::worker::run::habb95f20da6cdfc3",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 860,
                                                                                  "cacheValue": 828,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "0139fe52",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h6884d1e4e280994e",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 828,
                                                                                      "cacheValue": 828,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "00e11a72",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h93d3bd81bbbb2d22",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 828,
                                                                                          "cacheValue": 828,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "022a228c",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 828,
                                                                                              "cacheValue": 828,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "022992fa",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 828,
                                                                                                  "cacheValue": 828,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "02299941",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 828,
                                                                                                      "cacheValue": 828,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "023150e3",
                                                                                                            "executable": "light-node",
                                                                                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                            "functionCategory": "nodeRust"
                                                                                                          },
                                                                                                          "value": 828,
                                                                                                          "cacheValue": 828,
                                                                                                          "frames": [
                                                                                                            {
                                                                                                              "name": {
                                                                                                                "offset": "00101603",
                                                                                                                "executable": "libc.so.6",
                                                                                                                "functionName": "__clone",
                                                                                                                "functionCategory": "systemLib"
                                                                                                              },
                                                                                                              "value": 828,
                                                                                                              "cacheValue": 828,
                                                                                                              "frames": []
                                                                                                            }
                                                                                                          ]
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    },
                                                                                    {
                                                                                      "name": "underThreshold",
                                                                                      "value": 32,
                                                                                      "cacheValue": 0
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        }
                                                                      ]
                                                                    }
                                                                  ]
                                                                }
                                                              ]
                                                            }
                                                          ]
                                                        }
                                                      ]
                                                    }
                                                  ]
                                                }
                                              ]
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            },
            {
              "name": "underThreshold",
              "value": 32,
              "cacheValue": 0
            }
          ]
        },
        {
          "name": "underThreshold",
          "value": 24,
          "cacheValue": 24
        }
      ]
    },
    {
      "name": {
        "offset": "000128dd",
        "executable": "libpthread.so.0",
        "functionName": "write",
        "functionCategory": "systemLib"
      },
      "value": 758588,
      "cacheValue": 646456,
      "frames": [
        {
          "name": {
            "offset": "02307af3",
            "executable": "light-node",
            "functionName": "<std::fs::File as std::io::Write>::write::hd34ff43e8818e1a4",
            "functionCategory": "nodeRust"
          },
          "value": 643116,
          "cacheValue": 642248,
          "frames": [
            {
              "name": {
                "offset": "001db39d",
                "executable": "light-node",
                "functionName": "std::io::buffered::bufwriter::BufWriter<W>::write_all_cold::h2c0fe4b14fe8cacd",
                "functionCategory": "nodeRust"
              },
              "value": 632732,
              "cacheValue": 631944,
              "frames": [
                {
                  "name": {
                    "offset": "01a14492",
                    "executable": "light-node",
                    "functionName": "<storage::commit_log::CommitLogs as storage::commit_log::CommitLogWithSchema<S>>::append::h7767befccfb3c62e",
                    "functionCategory": "nodeRust"
                  },
                  "value": 632732,
                  "cacheValue": 631944,
                  "frames": [
                    {
                      "name": {
                        "offset": "01a058aa",
                        "executable": "light-node",
                        "functionName": "storage::block_storage::BlockStorage::put_block_json_data::hdc9adbea9c766190",
                        "functionCategory": "nodeRust"
                      },
                      "value": 632732,
                      "cacheValue": 631944,
                      "frames": [
                        {
                          "name": {
                            "offset": "01a3d623",
                            "executable": "light-node",
                            "functionName": "storage::store_applied_block_result::h7e45318485da314a",
                            "functionCategory": "nodeRust"
                          },
                          "value": 632732,
                          "cacheValue": 631944,
                          "frames": [
                            {
                              "name": {
                                "offset": "01313e9a",
                                "executable": "light-node",
                                "functionName": "shell_automaton::service::storage_service::StorageServiceDefault::run_worker::h32bbe7ed768e960e",
                                "functionCategory": "nodeRust"
                              },
                              "value": 632732,
                              "cacheValue": 631944,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "0138de7d",
                                    "executable": "light-node",
                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h47962be7f657fa3a",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 632732,
                                  "cacheValue": 631944,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "00ed0494",
                                        "executable": "light-node",
                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h8a52fef8fbba4f4f",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 632732,
                                      "cacheValue": 631944,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "023150e3",
                                            "executable": "light-node",
                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 632732,
                                          "cacheValue": 631944,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "00101603",
                                                "executable": "libc.so.6",
                                                "functionName": "__clone",
                                                "functionCategory": "systemLib"
                                              },
                                              "value": 632732,
                                              "cacheValue": 631944,
                                              "frames": []
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            },
            {
              "name": {
                "offset": "01a144f1",
                "executable": "light-node",
                "functionName": "<storage::commit_log::CommitLogs as storage::commit_log::CommitLogWithSchema<S>>::append::h7767befccfb3c62e",
                "functionCategory": "nodeRust"
              },
              "value": 3788,
              "cacheValue": 3788,
              "frames": [
                {
                  "name": {
                    "offset": "01a0537a",
                    "executable": "light-node",
                    "functionName": "storage::block_storage::BlockStorage::put_block_header::hb5a39ede6022474c",
                    "functionCategory": "nodeRust"
                  },
                  "value": 3788,
                  "cacheValue": 3788,
                  "frames": [
                    {
                      "name": {
                        "offset": "01310436",
                        "executable": "light-node",
                        "functionName": "shell_automaton::service::storage_service::StorageServiceDefault::run_worker::h32bbe7ed768e960e",
                        "functionCategory": "nodeRust"
                      },
                      "value": 3788,
                      "cacheValue": 3788,
                      "frames": [
                        {
                          "name": {
                            "offset": "0138de7d",
                            "executable": "light-node",
                            "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h47962be7f657fa3a",
                            "functionCategory": "nodeRust"
                          },
                          "value": 3788,
                          "cacheValue": 3788,
                          "frames": [
                            {
                              "name": {
                                "offset": "00ed0494",
                                "executable": "light-node",
                                "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h8a52fef8fbba4f4f",
                                "functionCategory": "nodeRust"
                              },
                              "value": 3788,
                              "cacheValue": 3788,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "023150e3",
                                    "executable": "light-node",
                                    "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 3788,
                                  "cacheValue": 3788,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "00101603",
                                        "executable": "libc.so.6",
                                        "functionName": "__clone",
                                        "functionCategory": "systemLib"
                                      },
                                      "value": 3788,
                                      "cacheValue": 3788,
                                      "frames": []
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            },
            {
              "name": {
                "offset": "00418ff4",
                "executable": "light-node",
                "functionName": "libflate::deflate::symbol::Encoder::encode::ha644887fb48b7bba",
                "functionCategory": "nodeRust"
              },
              "value": 3104,
              "cacheValue": 3104,
              "frames": [
                {
                  "name": {
                    "offset": "00422280",
                    "executable": "light-node",
                    "functionName": "libflate::deflate::encode::Block<E>::flush::h2a68eac7e402104c",
                    "functionCategory": "nodeRust"
                  },
                  "value": 3104,
                  "cacheValue": 3104,
                  "frames": [
                    {
                      "name": {
                        "offset": "00420ea1",
                        "executable": "light-node",
                        "functionName": "std::io::Write::write_all::h7abaca3a152f92b3",
                        "functionCategory": "nodeRust"
                      },
                      "value": 3104,
                      "cacheValue": 3104,
                      "frames": [
                        {
                          "name": {
                            "offset": "00436563",
                            "executable": "light-node",
                            "functionName": "std::io::copy::stack_buffer_copy::h208da19654cd5bb6",
                            "functionCategory": "nodeRust"
                          },
                          "value": 3104,
                          "cacheValue": 3104,
                          "frames": [
                            {
                              "name": {
                                "offset": "0041f828",
                                "executable": "light-node",
                                "functionName": "logging::file::FileAppender::compress::hecd7d51c949251ae",
                                "functionCategory": "nodeRust"
                              },
                              "value": 3104,
                              "cacheValue": 3104,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "0041b937",
                                    "executable": "light-node",
                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::hf7e13cebae2ad0bc",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 3104,
                                  "cacheValue": 3104,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "0042fca4",
                                        "executable": "light-node",
                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h56afea5c00971153",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 3104,
                                      "cacheValue": 3104,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "023150e3",
                                            "executable": "light-node",
                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 3104,
                                          "cacheValue": 3104,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "00101603",
                                                "executable": "libc.so.6",
                                                "functionName": "__clone",
                                                "functionCategory": "systemLib"
                                              },
                                              "value": 3104,
                                              "cacheValue": 3104,
                                              "frames": []
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            },
            {
              "name": {
                "offset": "0041911d",
                "executable": "light-node",
                "functionName": "libflate::deflate::symbol::Encoder::encode::ha644887fb48b7bba",
                "functionCategory": "nodeRust"
              },
              "value": 2008,
              "cacheValue": 2008,
              "frames": [
                {
                  "name": {
                    "offset": "00422280",
                    "executable": "light-node",
                    "functionName": "libflate::deflate::encode::Block<E>::flush::h2a68eac7e402104c",
                    "functionCategory": "nodeRust"
                  },
                  "value": 2008,
                  "cacheValue": 2008,
                  "frames": [
                    {
                      "name": {
                        "offset": "00420ea1",
                        "executable": "light-node",
                        "functionName": "std::io::Write::write_all::h7abaca3a152f92b3",
                        "functionCategory": "nodeRust"
                      },
                      "value": 2008,
                      "cacheValue": 2008,
                      "frames": [
                        {
                          "name": {
                            "offset": "00436563",
                            "executable": "light-node",
                            "functionName": "std::io::copy::stack_buffer_copy::h208da19654cd5bb6",
                            "functionCategory": "nodeRust"
                          },
                          "value": 2008,
                          "cacheValue": 2008,
                          "frames": [
                            {
                              "name": {
                                "offset": "0041f828",
                                "executable": "light-node",
                                "functionName": "logging::file::FileAppender::compress::hecd7d51c949251ae",
                                "functionCategory": "nodeRust"
                              },
                              "value": 2008,
                              "cacheValue": 2008,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "0041b937",
                                    "executable": "light-node",
                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::hf7e13cebae2ad0bc",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 2008,
                                  "cacheValue": 2008,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "0042fca4",
                                        "executable": "light-node",
                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h56afea5c00971153",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 2008,
                                      "cacheValue": 2008,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "023150e3",
                                            "executable": "light-node",
                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 2008,
                                          "cacheValue": 2008,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "00101603",
                                                "executable": "libc.so.6",
                                                "functionName": "__clone",
                                                "functionCategory": "systemLib"
                                              },
                                              "value": 2008,
                                              "cacheValue": 2008,
                                              "frames": []
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            },
            {
              "name": {
                "offset": "004190ce",
                "executable": "light-node",
                "functionName": "libflate::deflate::symbol::Encoder::encode::ha644887fb48b7bba",
                "functionCategory": "nodeRust"
              },
              "value": 960,
              "cacheValue": 960,
              "frames": [
                {
                  "name": {
                    "offset": "00422280",
                    "executable": "light-node",
                    "functionName": "libflate::deflate::encode::Block<E>::flush::h2a68eac7e402104c",
                    "functionCategory": "nodeRust"
                  },
                  "value": 960,
                  "cacheValue": 960,
                  "frames": [
                    {
                      "name": {
                        "offset": "00420ea1",
                        "executable": "light-node",
                        "functionName": "std::io::Write::write_all::h7abaca3a152f92b3",
                        "functionCategory": "nodeRust"
                      },
                      "value": 960,
                      "cacheValue": 960,
                      "frames": [
                        {
                          "name": {
                            "offset": "00436563",
                            "executable": "light-node",
                            "functionName": "std::io::copy::stack_buffer_copy::h208da19654cd5bb6",
                            "functionCategory": "nodeRust"
                          },
                          "value": 960,
                          "cacheValue": 960,
                          "frames": [
                            {
                              "name": {
                                "offset": "0041f828",
                                "executable": "light-node",
                                "functionName": "logging::file::FileAppender::compress::hecd7d51c949251ae",
                                "functionCategory": "nodeRust"
                              },
                              "value": 960,
                              "cacheValue": 960,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "0041b937",
                                    "executable": "light-node",
                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::hf7e13cebae2ad0bc",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 960,
                                  "cacheValue": 960,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "0042fca4",
                                        "executable": "light-node",
                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h56afea5c00971153",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 960,
                                      "cacheValue": 960,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "023150e3",
                                            "executable": "light-node",
                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 960,
                                          "cacheValue": 960,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "00101603",
                                                "executable": "libc.so.6",
                                                "functionName": "__clone",
                                                "functionCategory": "systemLib"
                                              },
                                              "value": 960,
                                              "cacheValue": 960,
                                              "frames": []
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            },
            {
              "name": "underThreshold",
              "value": 524,
              "cacheValue": 444
            }
          ]
        },
        {
          "name": {
            "offset": "022feac6",
            "executable": "light-node",
            "functionName": "<&std::os::unix::net::stream::UnixStream as std::io::Write>::write::h58f9eba4b251ba9f",
            "functionCategory": "nodeRust"
          },
          "value": 104608,
          "cacheValue": 0,
          "frames": [
            {
              "name": {
                "offset": "022aa3c7",
                "executable": "light-node",
                "functionName": "tokio::io::driver::registration::Registration::poll_write_io::h4db4404b43e2d046",
                "functionCategory": "nodeRust"
              },
              "value": 104608,
              "cacheValue": 0,
              "frames": [
                {
                  "name": {
                    "offset": "022951bc",
                    "executable": "light-node",
                    "functionName": "<tokio::net::unix::split_owned::OwnedWriteHalf as tokio::io::async_write::AsyncWrite>::poll_write::h933749bc07344d88",
                    "functionCategory": "nodeRust"
                  },
                  "value": 104608,
                  "cacheValue": 0,
                  "frames": [
                    {
                      "name": {
                        "offset": "007fe519",
                        "executable": "light-node",
                        "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::hf2a17a9d049978e3",
                        "functionCategory": "nodeRust"
                      },
                      "value": 100352,
                      "cacheValue": 0,
                      "frames": [
                        {
                          "name": {
                            "offset": "007c890a",
                            "executable": "light-node",
                            "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h73b1d32cc6e58446",
                            "functionCategory": "nodeRust"
                          },
                          "value": 100352,
                          "cacheValue": 0,
                          "frames": [
                            {
                              "name": {
                                "offset": "007f1315",
                                "executable": "light-node",
                                "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::hd512ea6ff56e70b2",
                                "functionCategory": "nodeRust"
                              },
                              "value": 100320,
                              "cacheValue": 0,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "007e506c",
                                    "executable": "light-node",
                                    "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::hbe540fe02292dde5",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 73568,
                                  "cacheValue": 0,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "007a9073",
                                        "executable": "light-node",
                                        "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h12a4497c4b3c35df",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 53792,
                                      "cacheValue": 0,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "005fa6c8",
                                            "executable": "light-node",
                                            "functionName": "<tokio::future::poll_fn::PollFn<F> as core::future::future::Future>::poll::h4bb26b926f86aae5",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 53792,
                                          "cacheValue": 0,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "007d0995",
                                                "executable": "light-node",
                                                "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h9009da781a4ec714",
                                                "functionCategory": "nodeRust"
                                              },
                                              "value": 53792,
                                              "cacheValue": 0,
                                              "frames": [
                                                {
                                                  "name": {
                                                    "offset": "0065de44",
                                                    "executable": "light-node",
                                                    "functionName": "<tokio::time::timeout::Timeout<T> as core::future::future::Future>::poll::hf3f7fcf3a9898d8b",
                                                    "functionCategory": "nodeRust"
                                                  },
                                                  "value": 53792,
                                                  "cacheValue": 0,
                                                  "frames": [
                                                    {
                                                      "name": {
                                                        "offset": "007b66b7",
                                                        "executable": "light-node",
                                                        "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h2d6367aa0d14c538",
                                                        "functionCategory": "nodeRust"
                                                      },
                                                      "value": 53792,
                                                      "cacheValue": 0,
                                                      "frames": [
                                                        {
                                                          "name": {
                                                            "offset": "0095a76d",
                                                            "executable": "light-node",
                                                            "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h7f3a4b7a4eb2dfb2",
                                                            "functionCategory": "nodeRust"
                                                          },
                                                          "value": 53792,
                                                          "cacheValue": 0,
                                                          "frames": [
                                                            {
                                                              "name": {
                                                                "offset": "006cee3e",
                                                                "executable": "light-node",
                                                                "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h4aa054e87161faf4",
                                                                "functionCategory": "nodeRust"
                                                              },
                                                              "value": 53792,
                                                              "cacheValue": 0,
                                                              "frames": [
                                                                {
                                                                  "name": {
                                                                    "offset": "022979b5",
                                                                    "executable": "light-node",
                                                                    "functionName": "std::thread::local::LocalKey<T>::with::hc00dafd9e6767220",
                                                                    "functionCategory": "nodeRust"
                                                                  },
                                                                  "value": 53792,
                                                                  "cacheValue": 0,
                                                                  "frames": [
                                                                    {
                                                                      "name": {
                                                                        "offset": "022b06b1",
                                                                        "executable": "light-node",
                                                                        "functionName": "tokio::runtime::thread_pool::worker::Context::run_task::hf312f91a8ef327f4",
                                                                        "functionCategory": "nodeRust"
                                                                      },
                                                                      "value": 53792,
                                                                      "cacheValue": 0,
                                                                      "frames": [
                                                                        {
                                                                          "name": {
                                                                            "offset": "022af4aa",
                                                                            "executable": "light-node",
                                                                            "functionName": "tokio::runtime::thread_pool::worker::Context::run::h6378a7a70d4c2172",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 52384,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "0229f145",
                                                                                "executable": "light-node",
                                                                                "functionName": "tokio::macros::scoped_tls::ScopedKey<T>::set::h414b9ab1e9afa14d",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 52384,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "022aef6b",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "tokio::runtime::thread_pool::worker::run::habb95f20da6cdfc3",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 52384,
                                                                                  "cacheValue": 0,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "0095a892",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h86f37711ea024eb4",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 43424,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "006cf462",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h73ed2b03505ee70f",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 43424,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "022a228c",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 43424,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "022992fa",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 43424,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "02299941",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 43424,
                                                                                                      "cacheValue": 0,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "023150e3",
                                                                                                            "executable": "light-node",
                                                                                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                            "functionCategory": "nodeRust"
                                                                                                          },
                                                                                                          "value": 43424,
                                                                                                          "cacheValue": 0,
                                                                                                          "frames": [
                                                                                                            {
                                                                                                              "name": {
                                                                                                                "offset": "00101603",
                                                                                                                "executable": "libc.so.6",
                                                                                                                "functionName": "__clone",
                                                                                                                "functionCategory": "systemLib"
                                                                                                              },
                                                                                                              "value": 43424,
                                                                                                              "cacheValue": 0,
                                                                                                              "frames": []
                                                                                                            }
                                                                                                          ]
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    },
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "0139fe52",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h6884d1e4e280994e",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 8960,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "00e11a72",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h93d3bd81bbbb2d22",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 8960,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "022a228c",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 8960,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "022992fa",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 8960,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "02299941",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 8960,
                                                                                                      "cacheValue": 0,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "023150e3",
                                                                                                            "executable": "light-node",
                                                                                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                            "functionCategory": "nodeRust"
                                                                                                          },
                                                                                                          "value": 8960,
                                                                                                          "cacheValue": 0,
                                                                                                          "frames": [
                                                                                                            {
                                                                                                              "name": {
                                                                                                                "offset": "00101603",
                                                                                                                "executable": "libc.so.6",
                                                                                                                "functionName": "__clone",
                                                                                                                "functionCategory": "systemLib"
                                                                                                              },
                                                                                                              "value": 8960,
                                                                                                              "cacheValue": 0,
                                                                                                              "frames": []
                                                                                                            }
                                                                                                          ]
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        },
                                                                        {
                                                                          "name": {
                                                                            "offset": "022afbab",
                                                                            "executable": "light-node",
                                                                            "functionName": "tokio::runtime::thread_pool::worker::Context::run::h6378a7a70d4c2172",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 1408,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "0229f145",
                                                                                "executable": "light-node",
                                                                                "functionName": "tokio::macros::scoped_tls::ScopedKey<T>::set::h414b9ab1e9afa14d",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 1408,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "022aef6b",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "tokio::runtime::thread_pool::worker::run::habb95f20da6cdfc3",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 1408,
                                                                                  "cacheValue": 0,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "0095a892",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h86f37711ea024eb4",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 1248,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "006cf462",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h73ed2b03505ee70f",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 1248,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "022a228c",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 1248,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "022992fa",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 1248,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "02299941",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 1248,
                                                                                                      "cacheValue": 0,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "023150e3",
                                                                                                            "executable": "light-node",
                                                                                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                            "functionCategory": "nodeRust"
                                                                                                          },
                                                                                                          "value": 1248,
                                                                                                          "cacheValue": 0,
                                                                                                          "frames": [
                                                                                                            {
                                                                                                              "name": {
                                                                                                                "offset": "00101603",
                                                                                                                "executable": "libc.so.6",
                                                                                                                "functionName": "__clone",
                                                                                                                "functionCategory": "systemLib"
                                                                                                              },
                                                                                                              "value": 1248,
                                                                                                              "cacheValue": 0,
                                                                                                              "frames": []
                                                                                                            }
                                                                                                          ]
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    },
                                                                                    {
                                                                                      "name": "underThreshold",
                                                                                      "value": 160,
                                                                                      "cacheValue": 0
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        }
                                                                      ]
                                                                    }
                                                                  ]
                                                                }
                                                              ]
                                                            }
                                                          ]
                                                        }
                                                      ]
                                                    }
                                                  ]
                                                }
                                              ]
                                            }
                                          ]
                                        }
                                      ]
                                    },
                                    {
                                      "name": {
                                        "offset": "006e36f5",
                                        "executable": "light-node",
                                        "functionName": "<tokio::future::maybe_done::MaybeDone<Fut> as core::future::future::Future>::poll::hc733355ff2938e55",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 19776,
                                      "cacheValue": 0,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "005fa67b",
                                            "executable": "light-node",
                                            "functionName": "<tokio::future::poll_fn::PollFn<F> as core::future::future::Future>::poll::h4bb26b926f86aae5",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 19776,
                                          "cacheValue": 0,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "007d0995",
                                                "executable": "light-node",
                                                "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h9009da781a4ec714",
                                                "functionCategory": "nodeRust"
                                              },
                                              "value": 19776,
                                              "cacheValue": 0,
                                              "frames": [
                                                {
                                                  "name": {
                                                    "offset": "0065de44",
                                                    "executable": "light-node",
                                                    "functionName": "<tokio::time::timeout::Timeout<T> as core::future::future::Future>::poll::hf3f7fcf3a9898d8b",
                                                    "functionCategory": "nodeRust"
                                                  },
                                                  "value": 19776,
                                                  "cacheValue": 0,
                                                  "frames": [
                                                    {
                                                      "name": {
                                                        "offset": "007b66b7",
                                                        "executable": "light-node",
                                                        "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h2d6367aa0d14c538",
                                                        "functionCategory": "nodeRust"
                                                      },
                                                      "value": 19776,
                                                      "cacheValue": 0,
                                                      "frames": [
                                                        {
                                                          "name": {
                                                            "offset": "0095a76d",
                                                            "executable": "light-node",
                                                            "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h7f3a4b7a4eb2dfb2",
                                                            "functionCategory": "nodeRust"
                                                          },
                                                          "value": 19776,
                                                          "cacheValue": 0,
                                                          "frames": [
                                                            {
                                                              "name": {
                                                                "offset": "006cee3e",
                                                                "executable": "light-node",
                                                                "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h4aa054e87161faf4",
                                                                "functionCategory": "nodeRust"
                                                              },
                                                              "value": 19776,
                                                              "cacheValue": 0,
                                                              "frames": [
                                                                {
                                                                  "name": {
                                                                    "offset": "022979b5",
                                                                    "executable": "light-node",
                                                                    "functionName": "std::thread::local::LocalKey<T>::with::hc00dafd9e6767220",
                                                                    "functionCategory": "nodeRust"
                                                                  },
                                                                  "value": 19776,
                                                                  "cacheValue": 0,
                                                                  "frames": [
                                                                    {
                                                                      "name": {
                                                                        "offset": "022b06b1",
                                                                        "executable": "light-node",
                                                                        "functionName": "tokio::runtime::thread_pool::worker::Context::run_task::hf312f91a8ef327f4",
                                                                        "functionCategory": "nodeRust"
                                                                      },
                                                                      "value": 19776,
                                                                      "cacheValue": 0,
                                                                      "frames": [
                                                                        {
                                                                          "name": {
                                                                            "offset": "022af4aa",
                                                                            "executable": "light-node",
                                                                            "functionName": "tokio::runtime::thread_pool::worker::Context::run::h6378a7a70d4c2172",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 19328,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "0229f145",
                                                                                "executable": "light-node",
                                                                                "functionName": "tokio::macros::scoped_tls::ScopedKey<T>::set::h414b9ab1e9afa14d",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 19328,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "022aef6b",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "tokio::runtime::thread_pool::worker::run::habb95f20da6cdfc3",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 19328,
                                                                                  "cacheValue": 0,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "0095a892",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h86f37711ea024eb4",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 15776,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "006cf462",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h73ed2b03505ee70f",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 15776,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "022a228c",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 15776,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "022992fa",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 15776,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "02299941",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 15776,
                                                                                                      "cacheValue": 0,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "023150e3",
                                                                                                            "executable": "light-node",
                                                                                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                            "functionCategory": "nodeRust"
                                                                                                          },
                                                                                                          "value": 15776,
                                                                                                          "cacheValue": 0,
                                                                                                          "frames": [
                                                                                                            {
                                                                                                              "name": {
                                                                                                                "offset": "00101603",
                                                                                                                "executable": "libc.so.6",
                                                                                                                "functionName": "__clone",
                                                                                                                "functionCategory": "systemLib"
                                                                                                              },
                                                                                                              "value": 15776,
                                                                                                              "cacheValue": 0,
                                                                                                              "frames": []
                                                                                                            }
                                                                                                          ]
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    },
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "0139fe52",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h6884d1e4e280994e",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 3552,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "00e11a72",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h93d3bd81bbbb2d22",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 3552,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "022a228c",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 3552,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "022992fa",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 3552,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "02299941",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 3552,
                                                                                                      "cacheValue": 0,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "023150e3",
                                                                                                            "executable": "light-node",
                                                                                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                            "functionCategory": "nodeRust"
                                                                                                          },
                                                                                                          "value": 3552,
                                                                                                          "cacheValue": 0,
                                                                                                          "frames": [
                                                                                                            {
                                                                                                              "name": {
                                                                                                                "offset": "00101603",
                                                                                                                "executable": "libc.so.6",
                                                                                                                "functionName": "__clone",
                                                                                                                "functionCategory": "systemLib"
                                                                                                              },
                                                                                                              "value": 3552,
                                                                                                              "cacheValue": 0,
                                                                                                              "frames": []
                                                                                                            }
                                                                                                          ]
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        },
                                                                        {
                                                                          "name": "underThreshold",
                                                                          "value": 448,
                                                                          "cacheValue": 0
                                                                        }
                                                                      ]
                                                                    }
                                                                  ]
                                                                }
                                                              ]
                                                            }
                                                          ]
                                                        }
                                                      ]
                                                    }
                                                  ]
                                                }
                                              ]
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                },
                                {
                                  "name": {
                                    "offset": "007d1cef",
                                    "executable": "light-node",
                                    "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h9159a4c733288ea6",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 26752,
                                  "cacheValue": 0,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "005fa57e",
                                        "executable": "light-node",
                                        "functionName": "<tokio::future::poll_fn::PollFn<F> as core::future::future::Future>::poll::h4bb26b926f86aae5",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 26752,
                                      "cacheValue": 0,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "007d0995",
                                            "executable": "light-node",
                                            "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h9009da781a4ec714",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 26752,
                                          "cacheValue": 0,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "0065de44",
                                                "executable": "light-node",
                                                "functionName": "<tokio::time::timeout::Timeout<T> as core::future::future::Future>::poll::hf3f7fcf3a9898d8b",
                                                "functionCategory": "nodeRust"
                                              },
                                              "value": 26752,
                                              "cacheValue": 0,
                                              "frames": [
                                                {
                                                  "name": {
                                                    "offset": "007b66b7",
                                                    "executable": "light-node",
                                                    "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h2d6367aa0d14c538",
                                                    "functionCategory": "nodeRust"
                                                  },
                                                  "value": 26752,
                                                  "cacheValue": 0,
                                                  "frames": [
                                                    {
                                                      "name": {
                                                        "offset": "0095a76d",
                                                        "executable": "light-node",
                                                        "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h7f3a4b7a4eb2dfb2",
                                                        "functionCategory": "nodeRust"
                                                      },
                                                      "value": 26752,
                                                      "cacheValue": 0,
                                                      "frames": [
                                                        {
                                                          "name": {
                                                            "offset": "006cee3e",
                                                            "executable": "light-node",
                                                            "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h4aa054e87161faf4",
                                                            "functionCategory": "nodeRust"
                                                          },
                                                          "value": 26752,
                                                          "cacheValue": 0,
                                                          "frames": [
                                                            {
                                                              "name": {
                                                                "offset": "022979b5",
                                                                "executable": "light-node",
                                                                "functionName": "std::thread::local::LocalKey<T>::with::hc00dafd9e6767220",
                                                                "functionCategory": "nodeRust"
                                                              },
                                                              "value": 26752,
                                                              "cacheValue": 0,
                                                              "frames": [
                                                                {
                                                                  "name": {
                                                                    "offset": "022b06b1",
                                                                    "executable": "light-node",
                                                                    "functionName": "tokio::runtime::thread_pool::worker::Context::run_task::hf312f91a8ef327f4",
                                                                    "functionCategory": "nodeRust"
                                                                  },
                                                                  "value": 26752,
                                                                  "cacheValue": 0,
                                                                  "frames": [
                                                                    {
                                                                      "name": {
                                                                        "offset": "022af4aa",
                                                                        "executable": "light-node",
                                                                        "functionName": "tokio::runtime::thread_pool::worker::Context::run::h6378a7a70d4c2172",
                                                                        "functionCategory": "nodeRust"
                                                                      },
                                                                      "value": 26656,
                                                                      "cacheValue": 0,
                                                                      "frames": [
                                                                        {
                                                                          "name": {
                                                                            "offset": "0229f145",
                                                                            "executable": "light-node",
                                                                            "functionName": "tokio::macros::scoped_tls::ScopedKey<T>::set::h414b9ab1e9afa14d",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 26656,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "022aef6b",
                                                                                "executable": "light-node",
                                                                                "functionName": "tokio::runtime::thread_pool::worker::run::habb95f20da6cdfc3",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 26656,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "0095a892",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h86f37711ea024eb4",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 21472,
                                                                                  "cacheValue": 0,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "006cf462",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h73ed2b03505ee70f",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 21472,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "022a228c",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 21472,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "022992fa",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 21472,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "02299941",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 21472,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "023150e3",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 21472,
                                                                                                      "cacheValue": 0,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "00101603",
                                                                                                            "executable": "libc.so.6",
                                                                                                            "functionName": "__clone",
                                                                                                            "functionCategory": "systemLib"
                                                                                                          },
                                                                                                          "value": 21472,
                                                                                                          "cacheValue": 0,
                                                                                                          "frames": []
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    }
                                                                                  ]
                                                                                },
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "0139fe52",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h6884d1e4e280994e",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 5184,
                                                                                  "cacheValue": 0,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "00e11a72",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h93d3bd81bbbb2d22",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 5184,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "022a228c",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 5184,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "022992fa",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 5184,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "02299941",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 5184,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "023150e3",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 5184,
                                                                                                      "cacheValue": 0,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "00101603",
                                                                                                            "executable": "libc.so.6",
                                                                                                            "functionName": "__clone",
                                                                                                            "functionCategory": "systemLib"
                                                                                                          },
                                                                                                          "value": 5184,
                                                                                                          "cacheValue": 0,
                                                                                                          "frames": []
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        }
                                                                      ]
                                                                    },
                                                                    {
                                                                      "name": "underThreshold",
                                                                      "value": 96,
                                                                      "cacheValue": 0
                                                                    }
                                                                  ]
                                                                }
                                                              ]
                                                            }
                                                          ]
                                                        }
                                                      ]
                                                    }
                                                  ]
                                                }
                                              ]
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            },
                            {
                              "name": "underThreshold",
                              "value": 32,
                              "cacheValue": 0
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": {
                        "offset": "00e502e9",
                        "executable": "light-node",
                        "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h839a19957f9a8264",
                        "functionCategory": "nodeRust"
                      },
                      "value": 4256,
                      "cacheValue": 0,
                      "frames": [
                        {
                          "name": {
                            "offset": "00e51268",
                            "executable": "light-node",
                            "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h9a3af0a9b57d7726",
                            "functionCategory": "nodeRust"
                          },
                          "value": 4224,
                          "cacheValue": 0,
                          "frames": [
                            {
                              "name": {
                                "offset": "0139ffbd",
                                "executable": "light-node",
                                "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h8de2c6d4faa97101",
                                "functionCategory": "nodeRust"
                              },
                              "value": 4224,
                              "cacheValue": 0,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "00e1107c",
                                    "executable": "light-node",
                                    "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h4c011eecbdd799ca",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 4224,
                                  "cacheValue": 0,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "022979b5",
                                        "executable": "light-node",
                                        "functionName": "std::thread::local::LocalKey<T>::with::hc00dafd9e6767220",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 4224,
                                      "cacheValue": 0,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "022b06b1",
                                            "executable": "light-node",
                                            "functionName": "tokio::runtime::thread_pool::worker::Context::run_task::hf312f91a8ef327f4",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 4224,
                                          "cacheValue": 0,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "022af4aa",
                                                "executable": "light-node",
                                                "functionName": "tokio::runtime::thread_pool::worker::Context::run::h6378a7a70d4c2172",
                                                "functionCategory": "nodeRust"
                                              },
                                              "value": 4224,
                                              "cacheValue": 0,
                                              "frames": [
                                                {
                                                  "name": {
                                                    "offset": "0229f145",
                                                    "executable": "light-node",
                                                    "functionName": "tokio::macros::scoped_tls::ScopedKey<T>::set::h414b9ab1e9afa14d",
                                                    "functionCategory": "nodeRust"
                                                  },
                                                  "value": 4224,
                                                  "cacheValue": 0,
                                                  "frames": [
                                                    {
                                                      "name": {
                                                        "offset": "022aef6b",
                                                        "executable": "light-node",
                                                        "functionName": "tokio::runtime::thread_pool::worker::run::habb95f20da6cdfc3",
                                                        "functionCategory": "nodeRust"
                                                      },
                                                      "value": 4224,
                                                      "cacheValue": 0,
                                                      "frames": [
                                                        {
                                                          "name": {
                                                            "offset": "0095a892",
                                                            "executable": "light-node",
                                                            "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h86f37711ea024eb4",
                                                            "functionCategory": "nodeRust"
                                                          },
                                                          "value": 3712,
                                                          "cacheValue": 0,
                                                          "frames": [
                                                            {
                                                              "name": {
                                                                "offset": "006cf462",
                                                                "executable": "light-node",
                                                                "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h73ed2b03505ee70f",
                                                                "functionCategory": "nodeRust"
                                                              },
                                                              "value": 3712,
                                                              "cacheValue": 0,
                                                              "frames": [
                                                                {
                                                                  "name": {
                                                                    "offset": "022a228c",
                                                                    "executable": "light-node",
                                                                    "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                    "functionCategory": "nodeRust"
                                                                  },
                                                                  "value": 3712,
                                                                  "cacheValue": 0,
                                                                  "frames": [
                                                                    {
                                                                      "name": {
                                                                        "offset": "022992fa",
                                                                        "executable": "light-node",
                                                                        "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                        "functionCategory": "nodeRust"
                                                                      },
                                                                      "value": 3712,
                                                                      "cacheValue": 0,
                                                                      "frames": [
                                                                        {
                                                                          "name": {
                                                                            "offset": "02299941",
                                                                            "executable": "light-node",
                                                                            "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 3712,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "023150e3",
                                                                                "executable": "light-node",
                                                                                "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 3712,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "00101603",
                                                                                    "executable": "libc.so.6",
                                                                                    "functionName": "__clone",
                                                                                    "functionCategory": "systemLib"
                                                                                  },
                                                                                  "value": 3712,
                                                                                  "cacheValue": 0,
                                                                                  "frames": []
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        }
                                                                      ]
                                                                    }
                                                                  ]
                                                                }
                                                              ]
                                                            }
                                                          ]
                                                        },
                                                        {
                                                          "name": {
                                                            "offset": "0139fe52",
                                                            "executable": "light-node",
                                                            "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h6884d1e4e280994e",
                                                            "functionCategory": "nodeRust"
                                                          },
                                                          "value": 512,
                                                          "cacheValue": 0,
                                                          "frames": [
                                                            {
                                                              "name": {
                                                                "offset": "00e11a72",
                                                                "executable": "light-node",
                                                                "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h93d3bd81bbbb2d22",
                                                                "functionCategory": "nodeRust"
                                                              },
                                                              "value": 512,
                                                              "cacheValue": 0,
                                                              "frames": [
                                                                {
                                                                  "name": {
                                                                    "offset": "022a228c",
                                                                    "executable": "light-node",
                                                                    "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                    "functionCategory": "nodeRust"
                                                                  },
                                                                  "value": 512,
                                                                  "cacheValue": 0,
                                                                  "frames": [
                                                                    {
                                                                      "name": {
                                                                        "offset": "022992fa",
                                                                        "executable": "light-node",
                                                                        "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                        "functionCategory": "nodeRust"
                                                                      },
                                                                      "value": 512,
                                                                      "cacheValue": 0,
                                                                      "frames": [
                                                                        {
                                                                          "name": {
                                                                            "offset": "02299941",
                                                                            "executable": "light-node",
                                                                            "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 512,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "023150e3",
                                                                                "executable": "light-node",
                                                                                "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 512,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "00101603",
                                                                                    "executable": "libc.so.6",
                                                                                    "functionName": "__clone",
                                                                                    "functionCategory": "systemLib"
                                                                                  },
                                                                                  "value": 512,
                                                                                  "cacheValue": 0,
                                                                                  "frames": []
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        }
                                                                      ]
                                                                    }
                                                                  ]
                                                                }
                                                              ]
                                                            }
                                                          ]
                                                        }
                                                      ]
                                                    }
                                                  ]
                                                }
                                              ]
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        },
                        {
                          "name": "underThreshold",
                          "value": 32,
                          "cacheValue": 0
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        },
        {
          "name": {
            "offset": "01d89236",
            "executable": "light-node",
            "functionName": "rocksdb::PosixWritableFile::Append(rocksdb::Slice const&, rocksdb::IOOptions const&, rocksdb::IODebugContext*)",
            "functionCategory": "nodeCpp"
          },
          "value": 10864,
          "cacheValue": 4208,
          "frames": []
        }
      ]
    },
    {
      "name": {
        "offset": "000131ed",
        "executable": "libpthread.so.0",
        "functionName": "__pread64",
        "functionCategory": "systemLib"
      },
      "value": 165404,
      "cacheValue": 162568,
      "frames": [
        {
          "name": {
            "offset": "01d8aa12",
            "executable": "light-node",
            "functionName": "rocksdb::PosixRandomAccessFile::Read(unsigned long, unsigned long, rocksdb::IOOptions const&, rocksdb::Slice*, char*, rocksdb::IODebugContext*) const",
            "functionCategory": "nodeCpp"
          },
          "value": 165404,
          "cacheValue": 162568,
          "frames": []
        }
      ]
    },
    {
      "name": {
        "offset": "000f876b",
        "executable": "libc.so.6",
        "functionName": "writev",
        "functionCategory": "systemLib"
      },
      "value": 100524,
      "cacheValue": 0,
      "frames": [
        {
          "name": {
            "offset": "02307783",
            "executable": "light-node",
            "functionName": "<&std::net::tcp::TcpStream as std::io::Write>::write_vectored::hcda0a3c6455d7c80",
            "functionCategory": "nodeRust"
          },
          "value": 100524,
          "cacheValue": 0,
          "frames": [
            {
              "name": {
                "offset": "022aa627",
                "executable": "light-node",
                "functionName": "tokio::io::driver::registration::Registration::poll_write_io::h7d0ca484d02fb42c",
                "functionCategory": "nodeRust"
              },
              "value": 100524,
              "cacheValue": 0,
              "frames": [
                {
                  "name": {
                    "offset": "022a5b35",
                    "executable": "light-node",
                    "functionName": "<tokio::net::tcp::stream::TcpStream as tokio::io::async_write::AsyncWrite>::poll_write_vectored::he574b286420111c3",
                    "functionCategory": "nodeRust"
                  },
                  "value": 100524,
                  "cacheValue": 0,
                  "frames": [
                    {
                      "name": {
                        "offset": "0078c738",
                        "executable": "light-node",
                        "functionName": "hyper::proto::h1::io::Buffered<T,B>::poll_flush::hd7c5cf0dee84295c",
                        "functionCategory": "nodeRust"
                      },
                      "value": 100524,
                      "cacheValue": 0,
                      "frames": [
                        {
                          "name": {
                            "offset": "006956bf",
                            "executable": "light-node",
                            "functionName": "hyper::proto::h1::conn::Conn<I,B,T>::poll_flush::h288fc4be2b976027",
                            "functionCategory": "nodeRust"
                          },
                          "value": 100524,
                          "cacheValue": 0,
                          "frames": [
                            {
                              "name": {
                                "offset": "005ec9c3",
                                "executable": "light-node",
                                "functionName": "hyper::proto::h1::dispatch::Dispatcher<D,Bs,I,T>::poll_loop::h40e1d345256f842a",
                                "functionCategory": "nodeRust"
                              },
                              "value": 69036,
                              "cacheValue": 0,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "005ea661",
                                    "executable": "light-node",
                                    "functionName": "hyper::proto::h1::dispatch::Dispatcher<D,Bs,I,T>::poll_catch::h0ae925c7a324338f",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 69036,
                                  "cacheValue": 0,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "00962bca",
                                        "executable": "light-node",
                                        "functionName": "<hyper::server::conn::upgrades::UpgradeableConnection<I,S,E> as core::future::future::Future>::poll::h29798eb7efc140e0",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 69036,
                                      "cacheValue": 0,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "00758cd1",
                                            "executable": "light-node",
                                            "functionName": "<hyper::server::conn::spawn_all::NewSvcTask<I,N,S,E,W> as core::future::future::Future>::poll::h75511f118733fcbd",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 69036,
                                          "cacheValue": 0,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "0095a65a",
                                                "executable": "light-node",
                                                "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h37d09748e26f8785",
                                                "functionCategory": "nodeRust"
                                              },
                                              "value": 69036,
                                              "cacheValue": 0,
                                              "frames": [
                                                {
                                                  "name": {
                                                    "offset": "006d1fc9",
                                                    "executable": "light-node",
                                                    "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::hc61ea0732ea65cc2",
                                                    "functionCategory": "nodeRust"
                                                  },
                                                  "value": 69036,
                                                  "cacheValue": 0,
                                                  "frames": [
                                                    {
                                                      "name": {
                                                        "offset": "022979b5",
                                                        "executable": "light-node",
                                                        "functionName": "std::thread::local::LocalKey<T>::with::hc00dafd9e6767220",
                                                        "functionCategory": "nodeRust"
                                                      },
                                                      "value": 69036,
                                                      "cacheValue": 0,
                                                      "frames": [
                                                        {
                                                          "name": {
                                                            "offset": "022b06b1",
                                                            "executable": "light-node",
                                                            "functionName": "tokio::runtime::thread_pool::worker::Context::run_task::hf312f91a8ef327f4",
                                                            "functionCategory": "nodeRust"
                                                          },
                                                          "value": 69036,
                                                          "cacheValue": 0,
                                                          "frames": [
                                                            {
                                                              "name": {
                                                                "offset": "022af4aa",
                                                                "executable": "light-node",
                                                                "functionName": "tokio::runtime::thread_pool::worker::Context::run::h6378a7a70d4c2172",
                                                                "functionCategory": "nodeRust"
                                                              },
                                                              "value": 69036,
                                                              "cacheValue": 0,
                                                              "frames": [
                                                                {
                                                                  "name": {
                                                                    "offset": "0229f145",
                                                                    "executable": "light-node",
                                                                    "functionName": "tokio::macros::scoped_tls::ScopedKey<T>::set::h414b9ab1e9afa14d",
                                                                    "functionCategory": "nodeRust"
                                                                  },
                                                                  "value": 69036,
                                                                  "cacheValue": 0,
                                                                  "frames": [
                                                                    {
                                                                      "name": {
                                                                        "offset": "022aef6b",
                                                                        "executable": "light-node",
                                                                        "functionName": "tokio::runtime::thread_pool::worker::run::habb95f20da6cdfc3",
                                                                        "functionCategory": "nodeRust"
                                                                      },
                                                                      "value": 69036,
                                                                      "cacheValue": 0,
                                                                      "frames": [
                                                                        {
                                                                          "name": {
                                                                            "offset": "0095a892",
                                                                            "executable": "light-node",
                                                                            "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h86f37711ea024eb4",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 61868,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "006cf462",
                                                                                "executable": "light-node",
                                                                                "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h73ed2b03505ee70f",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 61868,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "022a228c",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 61868,
                                                                                  "cacheValue": 0,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "022992fa",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 61868,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "02299941",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 61868,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "023150e3",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 61868,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "00101603",
                                                                                                    "executable": "libc.so.6",
                                                                                                    "functionName": "__clone",
                                                                                                    "functionCategory": "systemLib"
                                                                                                  },
                                                                                                  "value": 61868,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": []
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        },
                                                                        {
                                                                          "name": {
                                                                            "offset": "0139fe52",
                                                                            "executable": "light-node",
                                                                            "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h6884d1e4e280994e",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 7168,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "00e11a72",
                                                                                "executable": "light-node",
                                                                                "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h93d3bd81bbbb2d22",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 7168,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "022a228c",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 7168,
                                                                                  "cacheValue": 0,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "022992fa",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 7168,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "02299941",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 7168,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "023150e3",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 7168,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "00101603",
                                                                                                    "executable": "libc.so.6",
                                                                                                    "functionName": "__clone",
                                                                                                    "functionCategory": "systemLib"
                                                                                                  },
                                                                                                  "value": 7168,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": []
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        }
                                                                      ]
                                                                    }
                                                                  ]
                                                                }
                                                              ]
                                                            }
                                                          ]
                                                        }
                                                      ]
                                                    }
                                                  ]
                                                }
                                              ]
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            },
                            {
                              "name": {
                                "offset": "005ea94b",
                                "executable": "light-node",
                                "functionName": "hyper::proto::h1::dispatch::Dispatcher<D,Bs,I,T>::poll_loop::h40e1d345256f842a",
                                "functionCategory": "nodeRust"
                              },
                              "value": 31488,
                              "cacheValue": 0,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "005ea661",
                                    "executable": "light-node",
                                    "functionName": "hyper::proto::h1::dispatch::Dispatcher<D,Bs,I,T>::poll_catch::h0ae925c7a324338f",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 31488,
                                  "cacheValue": 0,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "00962bca",
                                        "executable": "light-node",
                                        "functionName": "<hyper::server::conn::upgrades::UpgradeableConnection<I,S,E> as core::future::future::Future>::poll::h29798eb7efc140e0",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 31488,
                                      "cacheValue": 0,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "00758cd1",
                                            "executable": "light-node",
                                            "functionName": "<hyper::server::conn::spawn_all::NewSvcTask<I,N,S,E,W> as core::future::future::Future>::poll::h75511f118733fcbd",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 31488,
                                          "cacheValue": 0,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "0095a65a",
                                                "executable": "light-node",
                                                "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h37d09748e26f8785",
                                                "functionCategory": "nodeRust"
                                              },
                                              "value": 31488,
                                              "cacheValue": 0,
                                              "frames": [
                                                {
                                                  "name": {
                                                    "offset": "006d1fc9",
                                                    "executable": "light-node",
                                                    "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::hc61ea0732ea65cc2",
                                                    "functionCategory": "nodeRust"
                                                  },
                                                  "value": 31488,
                                                  "cacheValue": 0,
                                                  "frames": [
                                                    {
                                                      "name": {
                                                        "offset": "022979b5",
                                                        "executable": "light-node",
                                                        "functionName": "std::thread::local::LocalKey<T>::with::hc00dafd9e6767220",
                                                        "functionCategory": "nodeRust"
                                                      },
                                                      "value": 31488,
                                                      "cacheValue": 0,
                                                      "frames": [
                                                        {
                                                          "name": {
                                                            "offset": "022b06b1",
                                                            "executable": "light-node",
                                                            "functionName": "tokio::runtime::thread_pool::worker::Context::run_task::hf312f91a8ef327f4",
                                                            "functionCategory": "nodeRust"
                                                          },
                                                          "value": 31488,
                                                          "cacheValue": 0,
                                                          "frames": [
                                                            {
                                                              "name": {
                                                                "offset": "022af4aa",
                                                                "executable": "light-node",
                                                                "functionName": "tokio::runtime::thread_pool::worker::Context::run::h6378a7a70d4c2172",
                                                                "functionCategory": "nodeRust"
                                                              },
                                                              "value": 31488,
                                                              "cacheValue": 0,
                                                              "frames": [
                                                                {
                                                                  "name": {
                                                                    "offset": "0229f145",
                                                                    "executable": "light-node",
                                                                    "functionName": "tokio::macros::scoped_tls::ScopedKey<T>::set::h414b9ab1e9afa14d",
                                                                    "functionCategory": "nodeRust"
                                                                  },
                                                                  "value": 31488,
                                                                  "cacheValue": 0,
                                                                  "frames": [
                                                                    {
                                                                      "name": {
                                                                        "offset": "022aef6b",
                                                                        "executable": "light-node",
                                                                        "functionName": "tokio::runtime::thread_pool::worker::run::habb95f20da6cdfc3",
                                                                        "functionCategory": "nodeRust"
                                                                      },
                                                                      "value": 31488,
                                                                      "cacheValue": 0,
                                                                      "frames": [
                                                                        {
                                                                          "name": {
                                                                            "offset": "0095a892",
                                                                            "executable": "light-node",
                                                                            "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h86f37711ea024eb4",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 27488,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "006cf462",
                                                                                "executable": "light-node",
                                                                                "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h73ed2b03505ee70f",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 27488,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "022a228c",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 27488,
                                                                                  "cacheValue": 0,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "022992fa",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 27488,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "02299941",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 27488,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "023150e3",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 27488,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "00101603",
                                                                                                    "executable": "libc.so.6",
                                                                                                    "functionName": "__clone",
                                                                                                    "functionCategory": "systemLib"
                                                                                                  },
                                                                                                  "value": 27488,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": []
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        },
                                                                        {
                                                                          "name": {
                                                                            "offset": "0139fe52",
                                                                            "executable": "light-node",
                                                                            "functionName": "tokio::runtime::task::core::CoreStage<T>::poll::h6884d1e4e280994e",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 4000,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "00e11a72",
                                                                                "executable": "light-node",
                                                                                "functionName": "tokio::runtime::task::harness::Harness<T,S>::poll::h93d3bd81bbbb2d22",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 4000,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "022a228c",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "tokio::runtime::blocking::pool::Inner::run::h61ed504425c9687e",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 4000,
                                                                                  "cacheValue": 0,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "022992fa",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::h865e36604d2ae921",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 4000,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "02299941",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::h3f4b79e0ec1b3566",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 4000,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "023150e3",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 4000,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "00101603",
                                                                                                    "executable": "libc.so.6",
                                                                                                    "functionName": "__clone",
                                                                                                    "functionCategory": "systemLib"
                                                                                                  },
                                                                                                  "value": 4000,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": []
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        }
                                                                                      ]
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        }
                                                                      ]
                                                                    }
                                                                  ]
                                                                }
                                                              ]
                                                            }
                                                          ]
                                                        }
                                                      ]
                                                    }
                                                  ]
                                                }
                                              ]
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    },
    {
      "name": {
        "offset": "00012d8c",
        "executable": "libpthread.so.0",
        "functionName": "__send",
        "functionCategory": "systemLib"
      },
      "value": 17696,
      "cacheValue": 0,
      "frames": [
        {
          "name": {
            "offset": "0230771a",
            "executable": "light-node",
            "functionName": "<&std::net::tcp::TcpStream as std::io::Write>::write::hdf61a03c6674acf1",
            "functionCategory": "nodeRust"
          },
          "value": 17696,
          "cacheValue": 0,
          "frames": [
            {
              "name": {
                "offset": "00cabc02",
                "executable": "light-node",
                "functionName": "shell_automaton::peer::peer_effects::peer_effects::h99dc7aaea7ac44f4",
                "functionCategory": "nodeRust"
              },
              "value": 17600,
              "cacheValue": 0,
              "frames": [
                {
                  "name": {
                    "offset": "00b9d73c",
                    "executable": "light-node",
                    "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                    "functionCategory": "nodeRust"
                  },
                  "value": 17600,
                  "cacheValue": 0,
                  "frames": [
                    {
                      "name": {
                        "offset": "00c7e364",
                        "executable": "light-node",
                        "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h589c3f8d80a47a8c",
                        "functionCategory": "nodeRust"
                      },
                      "value": 17600,
                      "cacheValue": 0,
                      "frames": [
                        {
                          "name": {
                            "offset": "00bd48c4",
                            "executable": "light-node",
                            "functionName": "shell_automaton::peer::chunk::write::peer_chunk_write_effects::peer_chunk_write_effects::hfd3646f13d088726",
                            "functionCategory": "nodeRust"
                          },
                          "value": 17504,
                          "cacheValue": 0,
                          "frames": [
                            {
                              "name": {
                                "offset": "00b9d82d",
                                "executable": "light-node",
                                "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                "functionCategory": "nodeRust"
                              },
                              "value": 17504,
                              "cacheValue": 0,
                              "frames": [
                                {
                                  "name": {
                                    "offset": "00c814aa",
                                    "executable": "light-node",
                                    "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h762ad1b0b5f33124",
                                    "functionCategory": "nodeRust"
                                  },
                                  "value": 17504,
                                  "cacheValue": 0,
                                  "frames": [
                                    {
                                      "name": {
                                        "offset": "00bd4857",
                                        "executable": "light-node",
                                        "functionName": "shell_automaton::peer::chunk::write::peer_chunk_write_effects::peer_chunk_write_effects::hfd3646f13d088726",
                                        "functionCategory": "nodeRust"
                                      },
                                      "value": 17504,
                                      "cacheValue": 0,
                                      "frames": [
                                        {
                                          "name": {
                                            "offset": "00b9d82d",
                                            "executable": "light-node",
                                            "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                            "functionCategory": "nodeRust"
                                          },
                                          "value": 17504,
                                          "cacheValue": 0,
                                          "frames": [
                                            {
                                              "name": {
                                                "offset": "00c7480a",
                                                "executable": "light-node",
                                                "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h020156cdb9d22308",
                                                "functionCategory": "nodeRust"
                                              },
                                              "value": 17504,
                                              "cacheValue": 0,
                                              "frames": [
                                                {
                                                  "name": {
                                                    "offset": "00bd48f5",
                                                    "executable": "light-node",
                                                    "functionName": "shell_automaton::peer::chunk::write::peer_chunk_write_effects::peer_chunk_write_effects::hfd3646f13d088726",
                                                    "functionCategory": "nodeRust"
                                                  },
                                                  "value": 17504,
                                                  "cacheValue": 0,
                                                  "frames": [
                                                    {
                                                      "name": {
                                                        "offset": "00b9d82d",
                                                        "executable": "light-node",
                                                        "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                        "functionCategory": "nodeRust"
                                                      },
                                                      "value": 17504,
                                                      "cacheValue": 0,
                                                      "frames": [
                                                        {
                                                          "name": {
                                                            "offset": "00c8586a",
                                                            "executable": "light-node",
                                                            "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h9c3603b3cead3b00",
                                                            "functionCategory": "nodeRust"
                                                          },
                                                          "value": 17504,
                                                          "cacheValue": 0,
                                                          "frames": [
                                                            {
                                                              "name": {
                                                                "offset": "00bd43e1",
                                                                "executable": "light-node",
                                                                "functionName": "shell_automaton::peer::binary_message::write::peer_binary_message_write_effects::peer_binary_message_write_effects::h63ef429bf2f8aefa",
                                                                "functionCategory": "nodeRust"
                                                              },
                                                              "value": 17504,
                                                              "cacheValue": 0,
                                                              "frames": [
                                                                {
                                                                  "name": {
                                                                    "offset": "00b9d817",
                                                                    "executable": "light-node",
                                                                    "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                    "functionCategory": "nodeRust"
                                                                  },
                                                                  "value": 17504,
                                                                  "cacheValue": 0,
                                                                  "frames": [
                                                                    {
                                                                      "name": {
                                                                        "offset": "00c8513a",
                                                                        "executable": "light-node",
                                                                        "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h99099f0ce4671553",
                                                                        "functionCategory": "nodeRust"
                                                                      },
                                                                      "value": 17504,
                                                                      "cacheValue": 0,
                                                                      "frames": [
                                                                        {
                                                                          "name": {
                                                                            "offset": "00c4eda2",
                                                                            "executable": "light-node",
                                                                            "functionName": "shell_automaton::peer::message::write::peer_message_write_effects::binary_message_write_init::hadde78a1fa550b64",
                                                                            "functionCategory": "nodeRust"
                                                                          },
                                                                          "value": 17504,
                                                                          "cacheValue": 0,
                                                                          "frames": [
                                                                            {
                                                                              "name": {
                                                                                "offset": "00c4f26d",
                                                                                "executable": "light-node",
                                                                                "functionName": "shell_automaton::peer::message::write::peer_message_write_effects::peer_message_write_effects::hde76482b9df1bc46",
                                                                                "functionCategory": "nodeRust"
                                                                              },
                                                                              "value": 17504,
                                                                              "cacheValue": 0,
                                                                              "frames": [
                                                                                {
                                                                                  "name": {
                                                                                    "offset": "00b9d80c",
                                                                                    "executable": "light-node",
                                                                                    "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                    "functionCategory": "nodeRust"
                                                                                  },
                                                                                  "value": 17504,
                                                                                  "cacheValue": 0,
                                                                                  "frames": [
                                                                                    {
                                                                                      "name": {
                                                                                        "offset": "00c81b9a",
                                                                                        "executable": "light-node",
                                                                                        "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h7bb9c0221baac1b2",
                                                                                        "functionCategory": "nodeRust"
                                                                                      },
                                                                                      "value": 17504,
                                                                                      "cacheValue": 0,
                                                                                      "frames": [
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "00c97572",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "shell_automaton::mempool::mempool_effects::mempool_effects::h725942768283914c",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 8512,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "00b9dca2",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 8512,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "00c7beb7",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h3d7c9818628e706c",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 8512,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "00c96506",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "shell_automaton::mempool::mempool_effects::mempool_effects::h725942768283914c",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 8512,
                                                                                                      "cacheValue": 0,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "00b9dca2",
                                                                                                            "executable": "light-node",
                                                                                                            "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                            "functionCategory": "nodeRust"
                                                                                                          },
                                                                                                          "value": 8512,
                                                                                                          "cacheValue": 0,
                                                                                                          "frames": [
                                                                                                            {
                                                                                                              "name": {
                                                                                                                "offset": "00c76237",
                                                                                                                "executable": "light-node",
                                                                                                                "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h0f9f380e79655c50",
                                                                                                                "functionCategory": "nodeRust"
                                                                                                              },
                                                                                                              "value": 8512,
                                                                                                              "cacheValue": 0,
                                                                                                              "frames": [
                                                                                                                {
                                                                                                                  "name": {
                                                                                                                    "offset": "00ca649e",
                                                                                                                    "executable": "light-node",
                                                                                                                    "functionName": "shell_automaton::mempool::validator::mempool_validator_effects::mempool_validator_effects::h7da8866fec155ad5",
                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                  },
                                                                                                                  "value": 8512,
                                                                                                                  "cacheValue": 0,
                                                                                                                  "frames": [
                                                                                                                    {
                                                                                                                      "name": {
                                                                                                                        "offset": "00b9dc97",
                                                                                                                        "executable": "light-node",
                                                                                                                        "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                      },
                                                                                                                      "value": 8512,
                                                                                                                      "cacheValue": 0,
                                                                                                                      "frames": [
                                                                                                                        {
                                                                                                                          "name": {
                                                                                                                            "offset": "00c7e209",
                                                                                                                            "executable": "light-node",
                                                                                                                            "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h58316f6e9e02d48a",
                                                                                                                            "functionCategory": "nodeRust"
                                                                                                                          },
                                                                                                                          "value": 8512,
                                                                                                                          "cacheValue": 0,
                                                                                                                          "frames": [
                                                                                                                            {
                                                                                                                              "name": {
                                                                                                                                "offset": "00bcfa77",
                                                                                                                                "executable": "light-node",
                                                                                                                                "functionName": "shell_automaton::protocol_runner::protocol_runner_effects::protocol_runner_effects::h279791a5d71ec38f",
                                                                                                                                "functionCategory": "nodeRust"
                                                                                                                              },
                                                                                                                              "value": 8512,
                                                                                                                              "cacheValue": 0,
                                                                                                                              "frames": [
                                                                                                                                {
                                                                                                                                  "name": {
                                                                                                                                    "offset": "00b9d68e",
                                                                                                                                    "executable": "light-node",
                                                                                                                                    "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                                  },
                                                                                                                                  "value": 8512,
                                                                                                                                  "cacheValue": 0,
                                                                                                                                  "frames": [
                                                                                                                                    {
                                                                                                                                      "name": {
                                                                                                                                        "offset": "00c83d64",
                                                                                                                                        "executable": "light-node",
                                                                                                                                        "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h8b5c8a5beac1b60b",
                                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                                      },
                                                                                                                                      "value": 8512,
                                                                                                                                      "cacheValue": 0,
                                                                                                                                      "frames": [
                                                                                                                                        {
                                                                                                                                          "name": {
                                                                                                                                            "offset": "00bd0db0",
                                                                                                                                            "executable": "light-node",
                                                                                                                                            "functionName": "shell_automaton::ShellAutomaton<Serv,Events>::make_progress::h04dd8ee4e8f2a479",
                                                                                                                                            "functionCategory": "nodeRust"
                                                                                                                                          },
                                                                                                                                          "value": 8512,
                                                                                                                                          "cacheValue": 0,
                                                                                                                                          "frames": [
                                                                                                                                            {
                                                                                                                                              "name": {
                                                                                                                                                "offset": "00b19a22",
                                                                                                                                                "executable": "light-node",
                                                                                                                                                "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::ha6ba004935f3fbc2",
                                                                                                                                                "functionCategory": "nodeRust"
                                                                                                                                              },
                                                                                                                                              "value": 8512,
                                                                                                                                              "cacheValue": 0,
                                                                                                                                              "frames": [
                                                                                                                                                {
                                                                                                                                                  "name": {
                                                                                                                                                    "offset": "00b4c03f",
                                                                                                                                                    "executable": "light-node",
                                                                                                                                                    "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::hf8cb803c841d06c5",
                                                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                                                  },
                                                                                                                                                  "value": 8512,
                                                                                                                                                  "cacheValue": 0,
                                                                                                                                                  "frames": [
                                                                                                                                                    {
                                                                                                                                                      "name": {
                                                                                                                                                        "offset": "023150e3",
                                                                                                                                                        "executable": "light-node",
                                                                                                                                                        "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                                                      },
                                                                                                                                                      "value": 8512,
                                                                                                                                                      "cacheValue": 0,
                                                                                                                                                      "frames": [
                                                                                                                                                        {
                                                                                                                                                          "name": {
                                                                                                                                                            "offset": "00101603",
                                                                                                                                                            "executable": "libc.so.6",
                                                                                                                                                            "functionName": "__clone",
                                                                                                                                                            "functionCategory": "systemLib"
                                                                                                                                                          },
                                                                                                                                                          "value": 8512,
                                                                                                                                                          "cacheValue": 0,
                                                                                                                                                          "frames": []
                                                                                                                                                        }
                                                                                                                                                      ]
                                                                                                                                                    }
                                                                                                                                                  ]
                                                                                                                                                }
                                                                                                                                              ]
                                                                                                                                            }
                                                                                                                                          ]
                                                                                                                                        }
                                                                                                                                      ]
                                                                                                                                    }
                                                                                                                                  ]
                                                                                                                                }
                                                                                                                              ]
                                                                                                                            }
                                                                                                                          ]
                                                                                                                        }
                                                                                                                      ]
                                                                                                                    }
                                                                                                                  ]
                                                                                                                }
                                                                                                              ]
                                                                                                            }
                                                                                                          ]
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        },
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "00c9705c",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "shell_automaton::mempool::mempool_effects::mempool_effects::h725942768283914c",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 7920,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "00b9dca2",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 7920,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "00c8effa",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::he6a6c44767d41ffb",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 7920,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "00cad1ba",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "shell_automaton::peer::message::read::peer_message_read_effects::peer_message_read_effects::h623cd0780b357e50",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 7920,
                                                                                                      "cacheValue": 0,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "00b9d801",
                                                                                                            "executable": "light-node",
                                                                                                            "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                            "functionCategory": "nodeRust"
                                                                                                          },
                                                                                                          "value": 7920,
                                                                                                          "cacheValue": 0,
                                                                                                          "frames": [
                                                                                                            {
                                                                                                              "name": {
                                                                                                                "offset": "00c88c0a",
                                                                                                                "executable": "light-node",
                                                                                                                "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::hb2c9ba45aa92485a",
                                                                                                                "functionCategory": "nodeRust"
                                                                                                              },
                                                                                                              "value": 7920,
                                                                                                              "cacheValue": 0,
                                                                                                              "frames": [
                                                                                                                {
                                                                                                                  "name": {
                                                                                                                    "offset": "00bd40db",
                                                                                                                    "executable": "light-node",
                                                                                                                    "functionName": "shell_automaton::peer::binary_message::read::peer_binary_message_read_effects::peer_binary_message_read_effects::h1660b0b046332fc4",
                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                  },
                                                                                                                  "value": 7920,
                                                                                                                  "cacheValue": 0,
                                                                                                                  "frames": [
                                                                                                                    {
                                                                                                                      "name": {
                                                                                                                        "offset": "00b9d822",
                                                                                                                        "executable": "light-node",
                                                                                                                        "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                      },
                                                                                                                      "value": 7920,
                                                                                                                      "cacheValue": 0,
                                                                                                                      "frames": [
                                                                                                                        {
                                                                                                                          "name": {
                                                                                                                            "offset": "00c781fd",
                                                                                                                            "executable": "light-node",
                                                                                                                            "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h1fae9aa88d22dd76",
                                                                                                                            "functionCategory": "nodeRust"
                                                                                                                          },
                                                                                                                          "value": 7920,
                                                                                                                          "cacheValue": 0,
                                                                                                                          "frames": [
                                                                                                                            {
                                                                                                                              "name": {
                                                                                                                                "offset": "00bd4141",
                                                                                                                                "executable": "light-node",
                                                                                                                                "functionName": "shell_automaton::peer::binary_message::read::peer_binary_message_read_effects::peer_binary_message_read_effects::h1660b0b046332fc4",
                                                                                                                                "functionCategory": "nodeRust"
                                                                                                                              },
                                                                                                                              "value": 7920,
                                                                                                                              "cacheValue": 0,
                                                                                                                              "frames": [
                                                                                                                                {
                                                                                                                                  "name": {
                                                                                                                                    "offset": "00b9d822",
                                                                                                                                    "executable": "light-node",
                                                                                                                                    "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                                  },
                                                                                                                                  "value": 7920,
                                                                                                                                  "cacheValue": 0,
                                                                                                                                  "frames": [
                                                                                                                                    {
                                                                                                                                      "name": {
                                                                                                                                        "offset": "00c821e4",
                                                                                                                                        "executable": "light-node",
                                                                                                                                        "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h7d64eaf7ddda70e0",
                                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                                      },
                                                                                                                                      "value": 7920,
                                                                                                                                      "cacheValue": 0,
                                                                                                                                      "frames": [
                                                                                                                                        {
                                                                                                                                          "name": {
                                                                                                                                            "offset": "00b41f22",
                                                                                                                                            "executable": "light-node",
                                                                                                                                            "functionName": "shell_automaton::peer::chunk::read::peer_chunk_read_effects::peer_chunk_read_effects::h4c312dc44cf04648",
                                                                                                                                            "functionCategory": "nodeRust"
                                                                                                                                          },
                                                                                                                                          "value": 7920,
                                                                                                                                          "cacheValue": 0,
                                                                                                                                          "frames": [
                                                                                                                                            {
                                                                                                                                              "name": {
                                                                                                                                                "offset": "00b9d838",
                                                                                                                                                "executable": "light-node",
                                                                                                                                                "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                                                                "functionCategory": "nodeRust"
                                                                                                                                              },
                                                                                                                                              "value": 7920,
                                                                                                                                              "cacheValue": 0,
                                                                                                                                              "frames": [
                                                                                                                                                {
                                                                                                                                                  "name": {
                                                                                                                                                    "offset": "00c80cea",
                                                                                                                                                    "executable": "light-node",
                                                                                                                                                    "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h734adff9dabef5f1",
                                                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                                                  },
                                                                                                                                                  "value": 7920,
                                                                                                                                                  "cacheValue": 0,
                                                                                                                                                  "frames": [
                                                                                                                                                    {
                                                                                                                                                      "name": {
                                                                                                                                                        "offset": "00b41f72",
                                                                                                                                                        "executable": "light-node",
                                                                                                                                                        "functionName": "shell_automaton::peer::chunk::read::peer_chunk_read_effects::peer_chunk_read_effects::h4c312dc44cf04648",
                                                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                                                      },
                                                                                                                                                      "value": 7920,
                                                                                                                                                      "cacheValue": 0,
                                                                                                                                                      "frames": [
                                                                                                                                                        {
                                                                                                                                                          "name": {
                                                                                                                                                            "offset": "00b9d838",
                                                                                                                                                            "executable": "light-node",
                                                                                                                                                            "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                                                                            "functionCategory": "nodeRust"
                                                                                                                                                          },
                                                                                                                                                          "value": 7920,
                                                                                                                                                          "cacheValue": 0,
                                                                                                                                                          "frames": [
                                                                                                                                                            {
                                                                                                                                                              "name": {
                                                                                                                                                                "offset": "00c765fa",
                                                                                                                                                                "executable": "light-node",
                                                                                                                                                                "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h1137afa6dbc2704d",
                                                                                                                                                                "functionCategory": "nodeRust"
                                                                                                                                                              },
                                                                                                                                                              "value": 7920,
                                                                                                                                                              "cacheValue": 0,
                                                                                                                                                              "frames": [
                                                                                                                                                                {
                                                                                                                                                                  "name": {
                                                                                                                                                                    "offset": "00cabf2c",
                                                                                                                                                                    "executable": "light-node",
                                                                                                                                                                    "functionName": "shell_automaton::peer::peer_effects::peer_effects::h99dc7aaea7ac44f4",
                                                                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                                                                  },
                                                                                                                                                                  "value": 7920,
                                                                                                                                                                  "cacheValue": 0,
                                                                                                                                                                  "frames": [
                                                                                                                                                                    {
                                                                                                                                                                      "name": {
                                                                                                                                                                        "offset": "00b9d73c",
                                                                                                                                                                        "executable": "light-node",
                                                                                                                                                                        "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                                                                      },
                                                                                                                                                                      "value": 7920,
                                                                                                                                                                      "cacheValue": 0,
                                                                                                                                                                      "frames": [
                                                                                                                                                                        {
                                                                                                                                                                          "name": {
                                                                                                                                                                            "offset": "00c7dcd4",
                                                                                                                                                                            "executable": "light-node",
                                                                                                                                                                            "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h5016745009da6b39",
                                                                                                                                                                            "functionCategory": "nodeRust"
                                                                                                                                                                          },
                                                                                                                                                                          "value": 7920,
                                                                                                                                                                          "cacheValue": 0,
                                                                                                                                                                          "frames": [
                                                                                                                                                                            {
                                                                                                                                                                              "name": {
                                                                                                                                                                                "offset": "00cac538",
                                                                                                                                                                                "executable": "light-node",
                                                                                                                                                                                "functionName": "shell_automaton::peer::peer_effects::peer_effects::h99dc7aaea7ac44f4",
                                                                                                                                                                                "functionCategory": "nodeRust"
                                                                                                                                                                              },
                                                                                                                                                                              "value": 7920,
                                                                                                                                                                              "cacheValue": 0,
                                                                                                                                                                              "frames": [
                                                                                                                                                                                {
                                                                                                                                                                                  "name": {
                                                                                                                                                                                    "offset": "00b9d73c",
                                                                                                                                                                                    "executable": "light-node",
                                                                                                                                                                                    "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                                                                                  },
                                                                                                                                                                                  "value": 7920,
                                                                                                                                                                                  "cacheValue": 0,
                                                                                                                                                                                  "frames": [
                                                                                                                                                                                    {
                                                                                                                                                                                      "name": {
                                                                                                                                                                                        "offset": "00c86e5e",
                                                                                                                                                                                        "executable": "light-node",
                                                                                                                                                                                        "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::ha60e1ab3f69f8ad2",
                                                                                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                                                                                      },
                                                                                                                                                                                      "value": 7920,
                                                                                                                                                                                      "cacheValue": 0,
                                                                                                                                                                                      "frames": [
                                                                                                                                                                                        {
                                                                                                                                                                                          "name": {
                                                                                                                                                                                            "offset": "00bd0d70",
                                                                                                                                                                                            "executable": "light-node",
                                                                                                                                                                                            "functionName": "shell_automaton::ShellAutomaton<Serv,Events>::make_progress::h04dd8ee4e8f2a479",
                                                                                                                                                                                            "functionCategory": "nodeRust"
                                                                                                                                                                                          },
                                                                                                                                                                                          "value": 7920,
                                                                                                                                                                                          "cacheValue": 0,
                                                                                                                                                                                          "frames": [
                                                                                                                                                                                            {
                                                                                                                                                                                              "name": {
                                                                                                                                                                                                "offset": "00b19a22",
                                                                                                                                                                                                "executable": "light-node",
                                                                                                                                                                                                "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::ha6ba004935f3fbc2",
                                                                                                                                                                                                "functionCategory": "nodeRust"
                                                                                                                                                                                              },
                                                                                                                                                                                              "value": 7920,
                                                                                                                                                                                              "cacheValue": 0,
                                                                                                                                                                                              "frames": [
                                                                                                                                                                                                {
                                                                                                                                                                                                  "name": {
                                                                                                                                                                                                    "offset": "00b4c03f",
                                                                                                                                                                                                    "executable": "light-node",
                                                                                                                                                                                                    "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::hf8cb803c841d06c5",
                                                                                                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                                                                                                  },
                                                                                                                                                                                                  "value": 7920,
                                                                                                                                                                                                  "cacheValue": 0,
                                                                                                                                                                                                  "frames": [
                                                                                                                                                                                                    {
                                                                                                                                                                                                      "name": {
                                                                                                                                                                                                        "offset": "023150e3",
                                                                                                                                                                                                        "executable": "light-node",
                                                                                                                                                                                                        "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                                                                                                      },
                                                                                                                                                                                                      "value": 7920,
                                                                                                                                                                                                      "cacheValue": 0,
                                                                                                                                                                                                      "frames": [
                                                                                                                                                                                                        {
                                                                                                                                                                                                          "name": {
                                                                                                                                                                                                            "offset": "00101603",
                                                                                                                                                                                                            "executable": "libc.so.6",
                                                                                                                                                                                                            "functionName": "__clone",
                                                                                                                                                                                                            "functionCategory": "systemLib"
                                                                                                                                                                                                          },
                                                                                                                                                                                                          "value": 7920,
                                                                                                                                                                                                          "cacheValue": 0,
                                                                                                                                                                                                          "frames": []
                                                                                                                                                                                                        }
                                                                                                                                                                                                      ]
                                                                                                                                                                                                    }
                                                                                                                                                                                                  ]
                                                                                                                                                                                                }
                                                                                                                                                                                              ]
                                                                                                                                                                                            }
                                                                                                                                                                                          ]
                                                                                                                                                                                        }
                                                                                                                                                                                      ]
                                                                                                                                                                                    }
                                                                                                                                                                                  ]
                                                                                                                                                                                }
                                                                                                                                                                              ]
                                                                                                                                                                            }
                                                                                                                                                                          ]
                                                                                                                                                                        }
                                                                                                                                                                      ]
                                                                                                                                                                    }
                                                                                                                                                                  ]
                                                                                                                                                                }
                                                                                                                                                              ]
                                                                                                                                                            }
                                                                                                                                                          ]
                                                                                                                                                        }
                                                                                                                                                      ]
                                                                                                                                                    }
                                                                                                                                                  ]
                                                                                                                                                }
                                                                                                                                              ]
                                                                                                                                            }
                                                                                                                                          ]
                                                                                                                                        }
                                                                                                                                      ]
                                                                                                                                    }
                                                                                                                                  ]
                                                                                                                                }
                                                                                                                              ]
                                                                                                                            }
                                                                                                                          ]
                                                                                                                        }
                                                                                                                      ]
                                                                                                                    }
                                                                                                                  ]
                                                                                                                }
                                                                                                              ]
                                                                                                            }
                                                                                                          ]
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        },
                                                                                        {
                                                                                          "name": {
                                                                                            "offset": "00c9449a",
                                                                                            "executable": "light-node",
                                                                                            "functionName": "shell_automaton::peer::remote_requests::block_operations_get::peer_remote_requests_block_operations_get_effects::peer_remote_requests_block_operations_get_effects::hdcc3a351be071ad9",
                                                                                            "functionCategory": "nodeRust"
                                                                                          },
                                                                                          "value": 896,
                                                                                          "cacheValue": 0,
                                                                                          "frames": [
                                                                                            {
                                                                                              "name": {
                                                                                                "offset": "00b9da41",
                                                                                                "executable": "light-node",
                                                                                                "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                "functionCategory": "nodeRust"
                                                                                              },
                                                                                              "value": 896,
                                                                                              "cacheValue": 0,
                                                                                              "frames": [
                                                                                                {
                                                                                                  "name": {
                                                                                                    "offset": "00c8ed65",
                                                                                                    "executable": "light-node",
                                                                                                    "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::he5c7b7662d4da3e2",
                                                                                                    "functionCategory": "nodeRust"
                                                                                                  },
                                                                                                  "value": 896,
                                                                                                  "cacheValue": 0,
                                                                                                  "frames": [
                                                                                                    {
                                                                                                      "name": {
                                                                                                        "offset": "00cacb1b",
                                                                                                        "executable": "light-node",
                                                                                                        "functionName": "shell_automaton::peer::peer_effects::peer_effects::h99dc7aaea7ac44f4",
                                                                                                        "functionCategory": "nodeRust"
                                                                                                      },
                                                                                                      "value": 896,
                                                                                                      "cacheValue": 0,
                                                                                                      "frames": [
                                                                                                        {
                                                                                                          "name": {
                                                                                                            "offset": "00b9d73c",
                                                                                                            "executable": "light-node",
                                                                                                            "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                            "functionCategory": "nodeRust"
                                                                                                          },
                                                                                                          "value": 896,
                                                                                                          "cacheValue": 0,
                                                                                                          "frames": [
                                                                                                            {
                                                                                                              "name": {
                                                                                                                "offset": "00c79039",
                                                                                                                "executable": "light-node",
                                                                                                                "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h26339d19fcc4ae55",
                                                                                                                "functionCategory": "nodeRust"
                                                                                                              },
                                                                                                              "value": 896,
                                                                                                              "cacheValue": 0,
                                                                                                              "frames": [
                                                                                                                {
                                                                                                                  "name": {
                                                                                                                    "offset": "00bd7401",
                                                                                                                    "executable": "light-node",
                                                                                                                    "functionName": "shell_automaton::storage::request::storage_request_effects::storage_request_effects::h35202a127214c605",
                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                  },
                                                                                                                  "value": 896,
                                                                                                                  "cacheValue": 0,
                                                                                                                  "frames": [
                                                                                                                    {
                                                                                                                      "name": {
                                                                                                                        "offset": "00b9dcad",
                                                                                                                        "executable": "light-node",
                                                                                                                        "functionName": "shell_automaton::effects::effects::hcdcbbff10f619d21",
                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                      },
                                                                                                                      "value": 896,
                                                                                                                      "cacheValue": 0,
                                                                                                                      "frames": [
                                                                                                                        {
                                                                                                                          "name": {
                                                                                                                            "offset": "00c83d64",
                                                                                                                            "executable": "light-node",
                                                                                                                            "functionName": "redux_rs::store::Store<State,Service,Action>::dispatch::h8b5c8a5beac1b60b",
                                                                                                                            "functionCategory": "nodeRust"
                                                                                                                          },
                                                                                                                          "value": 896,
                                                                                                                          "cacheValue": 0,
                                                                                                                          "frames": [
                                                                                                                            {
                                                                                                                              "name": {
                                                                                                                                "offset": "00bd0db0",
                                                                                                                                "executable": "light-node",
                                                                                                                                "functionName": "shell_automaton::ShellAutomaton<Serv,Events>::make_progress::h04dd8ee4e8f2a479",
                                                                                                                                "functionCategory": "nodeRust"
                                                                                                                              },
                                                                                                                              "value": 896,
                                                                                                                              "cacheValue": 0,
                                                                                                                              "frames": [
                                                                                                                                {
                                                                                                                                  "name": {
                                                                                                                                    "offset": "00b19a22",
                                                                                                                                    "executable": "light-node",
                                                                                                                                    "functionName": "std::sys_common::backtrace::__rust_begin_short_backtrace::ha6ba004935f3fbc2",
                                                                                                                                    "functionCategory": "nodeRust"
                                                                                                                                  },
                                                                                                                                  "value": 896,
                                                                                                                                  "cacheValue": 0,
                                                                                                                                  "frames": [
                                                                                                                                    {
                                                                                                                                      "name": {
                                                                                                                                        "offset": "00b4c03f",
                                                                                                                                        "executable": "light-node",
                                                                                                                                        "functionName": "core::ops::function::FnOnce::call_once{{vtable.shim}}::hf8cb803c841d06c5",
                                                                                                                                        "functionCategory": "nodeRust"
                                                                                                                                      },
                                                                                                                                      "value": 896,
                                                                                                                                      "cacheValue": 0,
                                                                                                                                      "frames": [
                                                                                                                                        {
                                                                                                                                          "name": {
                                                                                                                                            "offset": "023150e3",
                                                                                                                                            "executable": "light-node",
                                                                                                                                            "functionName": "std::sys::unix::thread::Thread::new::thread_start::ha678a8b0caec8f55",
                                                                                                                                            "functionCategory": "nodeRust"
                                                                                                                                          },
                                                                                                                                          "value": 896,
                                                                                                                                          "cacheValue": 0,
                                                                                                                                          "frames": [
                                                                                                                                            {
                                                                                                                                              "name": {
                                                                                                                                                "offset": "00101603",
                                                                                                                                                "executable": "libc.so.6",
                                                                                                                                                "functionName": "__clone",
                                                                                                                                                "functionCategory": "systemLib"
                                                                                                                                              },
                                                                                                                                              "value": 896,
                                                                                                                                              "cacheValue": 0,
                                                                                                                                              "frames": []
                                                                                                                                            }
                                                                                                                                          ]
                                                                                                                                        }
                                                                                                                                      ]
                                                                                                                                    }
                                                                                                                                  ]
                                                                                                                                }
                                                                                                                              ]
                                                                                                                            }
                                                                                                                          ]
                                                                                                                        }
                                                                                                                      ]
                                                                                                                    }
                                                                                                                  ]
                                                                                                                }
                                                                                                              ]
                                                                                                            }
                                                                                                          ]
                                                                                                        }
                                                                                                      ]
                                                                                                    }
                                                                                                  ]
                                                                                                }
                                                                                              ]
                                                                                            }
                                                                                          ]
                                                                                        },
                                                                                        {
                                                                                          "name": "underThreshold",
                                                                                          "value": 176,
                                                                                          "cacheValue": 0
                                                                                        }
                                                                                      ]
                                                                                    }
                                                                                  ]
                                                                                }
                                                                              ]
                                                                            }
                                                                          ]
                                                                        }
                                                                      ]
                                                                    }
                                                                  ]
                                                                }
                                                              ]
                                                            }
                                                          ]
                                                        }
                                                      ]
                                                    }
                                                  ]
                                                }
                                              ]
                                            }
                                          ]
                                        }
                                      ]
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        },
                        {
                          "name": "underThreshold",
                          "value": 96,
                          "cacheValue": 0
                        }
                      ]
                    }
                  ]
                }
              ]
            },
            {
              "name": "underThreshold",
              "value": 96,
              "cacheValue": 0
            }
          ]
        }
      ]
    },
    {
      "name": {
        "offset": "000f2a3d",
        "executable": "libc.so.6",
        "functionName": "__libc_write",
        "functionCategory": "systemLib"
      },
      "value": 4276,
      "cacheValue": 4236,
      "frames": [
        {
          "name": {
            "offset": "00082055",
            "executable": "libc.so.6",
            "functionName": "_IO_new_file_write",
            "functionCategory": "systemLib"
          },
          "value": 4276,
          "cacheValue": 4236,
          "frames": [
            {
              "name": {
                "offset": "00081399",
                "executable": "libc.so.6",
                "functionName": "new_do_write",
                "functionCategory": "systemLib"
              },
              "value": 4276,
              "cacheValue": 4236,
              "frames": [
                {
                  "name": {
                    "offset": "000827aa",
                    "executable": "libc.so.6",
                    "functionName": "_IO_file_xsputn@@GLIBC_2.2.5",
                    "functionCategory": "systemLib"
                  },
                  "value": 3448,
                  "cacheValue": 3424,
                  "frames": [
                    {
                      "name": {
                        "offset": "00076f83",
                        "executable": "libc.so.6",
                        "functionName": "__GI_fwrite",
                        "functionCategory": "systemLib"
                      },
                      "value": 3448,
                      "cacheValue": 3424,
                      "frames": [
                        {
                          "name": {
                            "offset": "01d77d11",
                            "executable": "light-node",
                            "functionName": "rocksdb::PosixLogger::Logv(char const*, __va_list_tag*)",
                            "functionCategory": "nodeCpp"
                          },
                          "value": 3448,
                          "cacheValue": 3424,
                          "frames": []
                        }
                      ]
                    }
                  ]
                },
                {
                  "name": {
                    "offset": "00083191",
                    "executable": "libc.so.6",
                    "functionName": "_IO_new_do_write",
                    "functionCategory": "systemLib"
                  },
                  "value": 828,
                  "cacheValue": 812,
                  "frames": []
                }
              ]
            }
          ]
        }
      ]
    },
    {
      "name": {
        "offset": "0008d280",
        "executable": "libc.so.6",
        "functionName": "free",
        "functionCategory": "systemLib"
      },
      "value": 3712,
      "cacheValue": 0,
      "frames": [
        {
          "name": {
            "offset": "01f44eb7",
            "executable": "light-node",
            "functionName": "rocksdb::BinarySearchIndexReader::~BinarySearchIndexReader()",
            "functionCategory": "nodeCpp"
          },
          "value": 2232,
          "cacheValue": 0,
          "frames": []
        },
        {
          "name": {
            "offset": "01c36433",
            "executable": "light-node",
            "functionName": "std::_Rb_tree<std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> >, std::pair<std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> > const, std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> > >, std::_Select1st<std::pair<std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> > const, std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> > > >, std::less<std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> > >, std::allocator<std::pair<std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> > const, std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> > > > >::_M_erase(std::_Rb_tree_node<std::pair<std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> > const, std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> > > >*)",
            "functionCategory": "nodeCpp"
          },
          "value": 992,
          "cacheValue": 0,
          "frames": []
        },
        {
          "name": "underThreshold",
          "value": 488,
          "cacheValue": 0
        }
      ]
    },
    {
      "name": {
        "offset": "00089422",
        "executable": "libc.so.6",
        "functionName": "_int_free",
        "functionCategory": "systemLib"
      },
      "value": 2116,
      "cacheValue": 0,
      "frames": [
        {
          "name": {
            "offset": "0008d2c3",
            "executable": "libc.so.6",
            "functionName": "free",
            "functionCategory": "systemLib"
          },
          "value": 2116,
          "cacheValue": 0,
          "frames": [
            {
              "name": {
                "offset": "01f44eb7",
                "executable": "light-node",
                "functionName": "rocksdb::BinarySearchIndexReader::~BinarySearchIndexReader()",
                "functionCategory": "nodeCpp"
              },
              "value": 2040,
              "cacheValue": 0,
              "frames": []
            },
            {
              "name": "underThreshold",
              "value": 76,
              "cacheValue": 0
            }
          ]
        }
      ]
    },
    {
      "name": {
        "offset": "001022fe",
        "executable": "libc.so.6",
        "functionName": "__GI_epoll_ctl",
        "functionCategory": "systemLib"
      },
      "value": 1956,
      "cacheValue": 0,
      "frames": [
        {
          "name": {
            "offset": "022b3cf4",
            "executable": "light-node",
            "functionName": "tokio::io::driver::Inner::add_source::h167019cc9c4fd6ad",
            "functionCategory": "nodeRust"
          },
          "value": 1380,
          "cacheValue": 0,
          "frames": [
            {
              "name": {
                "offset": "022b9f71",
                "executable": "light-node",
                "functionName": "tokio::io::poll_evented::PollEvented<E>::new::h0c1426f73340f045",
                "functionCategory": "nodeRust"
              },
              "value": 1380,
              "cacheValue": 0,
              "frames": [
                {
                  "name": {
                    "offset": "022aaad5",
                    "executable": "light-node",
                    "functionName": "tokio::net::unix::stream::UnixStream::new::h75841f4e06d96660",
                    "functionCategory": "nodeRust"
                  },
                  "value": 1380,
                  "cacheValue": 0,
                  "frames": [
                    {
                      "name": {
                        "offset": "007f58e2",
                        "executable": "light-node",
                        "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::hda958a47c0757418",
                        "functionCategory": "nodeRust"
                      },
                      "value": 1364,
                      "cacheValue": 0,
                      "frames": [
                        {
                          "name": {
                            "offset": "007f102c",
                            "executable": "light-node",
                            "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::hd512ea6ff56e70b2",
                            "functionCategory": "nodeRust"
                          },
                          "value": 600,
                          "cacheValue": 0,
                          "frames": []
                        },
                        {
                          "name": {
                            "offset": "008003a8",
                            "executable": "light-node",
                            "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::hfdb826ba0fa532fd",
                            "functionCategory": "nodeRust"
                          },
                          "value": 592,
                          "cacheValue": 0,
                          "frames": [
                            {
                              "name": {
                                "offset": "007c7a0c",
                                "executable": "light-node",
                                "functionName": "<core::future::from_generator::GenFuture<T> as core::future::future::Future>::poll::h6d39d73bd1902842",
                                "functionCategory": "nodeRust"
                              },
                              "value": 588,
                              "cacheValue": 0,
                              "frames": []
                            },
                            {
                              "name": "underThreshold",
                              "value": 4,
                              "cacheValue": 0
                            }
                          ]
                        },
                        {
                          "name": "underThreshold",
                          "value": 172,
                          "cacheValue": 0
                        }
                      ]
                    },
                    {
                      "name": "underThreshold",
                      "value": 16,
                      "cacheValue": 0
                    }
                  ]
                }
              ]
            }
          ]
        },
        {
          "name": "underThreshold",
          "value": 576,
          "cacheValue": 0
        }
      ]
    },
    {
      "name": {
        "offset": "000f945d",
        "executable": "libc.so.6",
        "functionName": "fdatasync",
        "functionCategory": "systemLib"
      },
      "value": 1776,
      "cacheValue": 112,
      "frames": [
        {
          "name": {
            "offset": "01d89696",
            "executable": "light-node",
            "functionName": "rocksdb::PosixWritableFile::Sync(rocksdb::IOOptions const&, rocksdb::IODebugContext*)",
            "functionCategory": "nodeCpp"
          },
          "value": 1664,
          "cacheValue": 96,
          "frames": []
        },
        {
          "name": "underThreshold",
          "value": 112,
          "cacheValue": 16
        }
      ]
    },
    {
      "name": {
        "offset": "000893b4",
        "executable": "libc.so.6",
        "functionName": "_int_free",
        "functionCategory": "systemLib"
      },
      "value": 1764,
      "cacheValue": 0,
      "frames": [
        {
          "name": {
            "offset": "0008d2c3",
            "executable": "libc.so.6",
            "functionName": "free",
            "functionCategory": "systemLib"
          },
          "value": 1764,
          "cacheValue": 0,
          "frames": [
            {
              "name": {
                "offset": "01f44eb7",
                "executable": "light-node",
                "functionName": "rocksdb::BinarySearchIndexReader::~BinarySearchIndexReader()",
                "functionCategory": "nodeCpp"
              },
              "value": 1740,
              "cacheValue": 0,
              "frames": []
            },
            {
              "name": "underThreshold",
              "value": 24,
              "cacheValue": 0
            }
          ]
        }
      ]
    },
    {
      "name": {
        "offset": "000130aa",
        "executable": "libpthread.so.0",
        "functionName": "__open64",
        "functionCategory": "systemLib"
      },
      "value": 1256,
      "cacheValue": 0,
      "frames": [
        {
          "name": {
            "offset": "0230f31f",
            "executable": "light-node",
            "functionName": "std::sys::unix::fs::File::open_c::h5e7f573337528e1d",
            "functionCategory": "nodeRust"
          },
          "value": 1256,
          "cacheValue": 0,
          "frames": []
        }
      ]
    },
    {
      "name": {
        "offset": "003c455c",
        "executable": "light-node",
        "functionName": "_rjem_je_extent_avail_remove",
        "functionCategory": "nodeCpp"
      },
      "value": 1240,
      "cacheValue": 0,
      "frames": []
    },
    {
      "name": {
        "offset": "001015f5",
        "executable": "libc.so.6",
        "functionName": "__clone",
        "functionCategory": "systemLib"
      },
      "value": 812,
      "cacheValue": 0,
      "frames": [
        {
          "name": {
            "offset": "00009b0b",
            "executable": "libpthread.so.0",
            "functionName": "pthread_create@@GLIBC_2.2.5",
            "functionCategory": "systemLib"
          },
          "value": 812,
          "cacheValue": 0,
          "frames": []
        }
      ]
    },
    {
      "name": {
        "offset": "01dfec23",
        "executable": "light-node",
        "functionName": "rocksdb::BlockBasedTable::~BlockBasedTable()",
        "functionCategory": "nodeCpp"
      },
      "value": 712,
      "cacheValue": 0,
      "frames": []
    },
    {
      "name": {
        "offset": "00088fc7",
        "executable": "libc.so.6",
        "functionName": "unlink_chunk.constprop.0",
        "functionCategory": "systemLib"
      },
      "value": 584,
      "cacheValue": 0,
      "frames": []
    },
    {
      "name": {
        "offset": "01ec3010",
        "executable": "light-node",
        "functionName": "rocksdb::LRUCache::GetHash(rocksdb::Cache::Handle*) const",
        "functionCategory": "nodeCpp"
      },
      "value": 532,
      "cacheValue": 0,
      "frames": []
    },
    {
      "name": "underThreshold",
      "value": 8680,
      "cacheValue": 3116
    }
  ]
};
