#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

use derive_more::Default;

#[derive(Default)]
struct Point2D<'a> {
    #[default(value=1)]
    pub x: i32,
    #[default(value=2, constant)]
    pub y: i32,
    #[default(value=3, function)]
    pub z: i32,

    #[default(value=10, function, constant)]
    pub a: i32,
    #[default(value=11, constant, function)]
    pub b: i32,

    #[default(value=10+10, function=init_c, constant=INIT_C)]
    pub c: i32,
    #[default(value=10+11, constant=INIT_D, function=init_d)]
    pub d: i32,

    #[default(value="ok", function=init_e)]
    pub e: &'a str,
    #[default(value="ok", function)]
    pub f: String,

    #[default(value = "localhost")]
    pub host: std::borrow::Cow<'static, str>,
}

#[derive(Default)]
struct Point3D (
    #[default(value=1)]
    pub i32,
    #[default(value=2, constant)]
    pub i32,
    #[default(value=3, function)]
    pub i32,
);

#[derive(Default)]
enum SimpleEnum {
    #[default]
    One,
    Two,
}

#[derive(Default)]
enum NamedStructEnum {
    None,
    #[default(value=Self::Point2D{x:1,y:2})]
    Point2D {
        x: i32,
        y: i32
    },
}

#[derive(Default)]
enum UnamedStructEnum {
    None,
    #[default(value=Self::Point2D(1,2))]
    Point2D (
        i32,
        i32
    ),
}

#[derive(Default)]
enum GenericEnum<T> {
    #[default]
    None,
    Some(T),
}


#[test]
fn named_struct_test() {
    assert_eq!(1, Point2D::default().x);

    assert_eq!(2, Point2D::default().y);
    assert_eq!(2, Point2D::DEFAULT_Y);

    assert_eq!(3, Point2D::default().z);
    assert_eq!(3, Point2D::default_z());

    assert_eq!(10, Point2D::default().a);
    assert_eq!(10, Point2D::DEFAULT_A);
    assert_eq!(10, Point2D::default_a());

    assert_eq!(11, Point2D::default().b);
    assert_eq!(11, Point2D::DEFAULT_B);
    assert_eq!(11, Point2D::default_b());

    assert_eq!(20, Point2D::default().c);
    assert_eq!(20, Point2D::INIT_C);
    assert_eq!(20, Point2D::init_c());

    assert_eq!(21, Point2D::default().d);
    assert_eq!(21, Point2D::INIT_D);
    assert_eq!(21, Point2D::init_d());
}


#[test]
fn unnamed_struct_test() {
    assert_eq!(1, Point3D::default().0);

    assert_eq!(2, Point3D::default().1);
    assert_eq!(2, Point3D::DEFAULT_1);

    assert_eq!(3, Point3D::default().2);
    assert_eq!(3, Point3D::default_2());
}


#[test]
fn enum_test() {
    assert!(matches!(SimpleEnum::default(), SimpleEnum::One));
}

#[test]
fn generic_enum_test() {
    assert!(matches!(GenericEnum::<i32>::default(), GenericEnum::None));
}


#[test]
fn named_enum_test() {
    assert!(matches!(NamedStructEnum::default(), NamedStructEnum::Point2D { x:1, y:2 }));
}


#[test]
fn unnamed_enum_test() {
    assert!(matches!(UnamedStructEnum::default(), UnamedStructEnum::Point2D( 1, 2 )));
}
