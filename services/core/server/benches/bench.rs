//! Micro-Performance Benchmarks. Endpoint performance can monitored in the Grafana dashboard.

use criterion::{Criterion, criterion_group, criterion_main};
use crypto::HashedPassword;
use models::{
    DbInitFlags, initialize_database,
    user_model::{NewUserModel, UserModel},
};

fn sample_benchmark(c: &mut Criterion) {
    initialize_database(DbInitFlags::NUKE);

    c.bench_function("Insert/Delete user", |b| {
        b.iter(|| {
            let user = UserModel::new(NewUserModel {
                name: String::from("asdf"),
                password_hash: String::from("asdf"),
            })
            .unwrap();

            UserModel::delete(user.id).unwrap();
        });
    });

    c.bench_function("Hash password", |b| {
        b.iter(|| {
            HashedPassword::new("asdf").unwrap();
        });
    });

    c.bench_function("Hash+Check password", |b| {
        b.iter(|| {
            let pw = HashedPassword::new("asdf").unwrap();

            pw.check("asdf");
            pw.check("asdf2");
        });
    });
}

criterion_group!(benches, sample_benchmark);
criterion_main!(benches);
