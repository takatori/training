use crate::{helper::safe_add, parser};
use std::{borrow::Cow, cmp::Ordering, collections::BTreeMap, mem};

type VarToType = BTreeMap<String, Option<parser::TypeExpr>>;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct TypeEnvStack {
    vars: BTreeMap<usize, VarToType>,
}

impl TypeEnvStack {
    fn new() -> TypeEnvStack {
        TypeEnvStack {
            vars: BTreeMap::new(),
        }
    }

    /// 型環境をpush
    fn push(&mut self, depth: usize) {
        self.vars.insert(depth, BTreeMap::new());
    }

    /// 型環境をpop
    fn pop(&mut self, depth: usize) -> Option<VarToType> {
        self.vars.remove(&depth)
    }

    /// スタックの最も上にある肩環境に変数と型を追加
    fn insert(&mut self, key: String, value: parser::TypeExpr) {
        if let Some(last) = self.vars.iter_mut().next_back() {
            last.1.insert(key, Some(value));
        }
    }

    fn get_mut(&mut self, key: &str) -> Option<(usize, &mut Option<parser::TypeExpr>)> {
        for (depth, elm) in self.vars.iter_mut().rev() {
            if let Some(e) = elm.get_mut(key) {
                return Some((*depth, e));
            }
        }
        None
    }
}

type TResult<'a> = Result<parser::TypeExpr, Cow<'a, str>>;

pub fn typing<'a>(expr: &parser::Expr, env: &mut TypeEnv, depth: usize) -> TResult<'a> {
    match expr {
        parser::Expr::App(e) => typing_app(e, env, depth),
        parser::Expr::QVal(e) => typing_qval(e, env, depth),
        parser::Expr::Free(e) => typing_free(e, env, depth),
        parser::Expr::If(e) => typing_if(e, env, depth),
        parser::Expr::Split(e) => typing_split(e, env, depth),
        parser::Expr::Var(e) => typing_var(e, env),
        parser::Expr::Let(e) => typing_let(e, env, depth),
    }
}

/// 修飾子付きの型付け
fn typeing_qval<'a>(expr: &parser::QValExpr, env: &mut TypeEnv, depth: usize) -> TResult<'a> {
    // プリミティブ型を計算
    let p = match &expr.val {
        parser::ValExpr::Bool(_) => parser::PrimType::Bool,
        parser::ValExpr::Pair(e1, e2) => {
            // 式e1とe2をtypingにより型付け
            let t1 = typing(e1, env, depth)?;
            let t2 = typing(e2, env, depth)?;

            // expr.qualがunであり、
            // e1かe2の型にlinが含まれていた場合、型付けエラー
            if expr.qual == parser::Qual::Un 
                && (t1.qual == parser::Qual::Lin || t2.qual == parser::Qual::Lin) {
                    return Err("un型のペア内でlin型を使用している".into());
                }

            // ペア型を返す
            parser::PrimType::Pair(Box::new(t1), Box::new(t2))
        }
        parser::ValExpr::Fun(e) => {
            // 関数の型付け
            // un型の関数内では、lin型の自由変数をキャプチャできないため
            // lin用の型環境を置き換え
            let env_prev = if expr.qual == parser::Qual::Un {
                Some(mem::take(&mut env.ev_lin))
            } else {
                None
            };

            // depthをインクリメントしてpush
            let mut depth = depth;
            safe_add(&mut depth, &1, || "変数スコープのネストが深すぎる")?;
            env.push(depth);
            env.insert(e.var.clone(), e.ty.clone());

            // 関数中の式を型付け
            let t = typing(&e.expr, env, depth)?;

            // 型環境をpopし、popした型環境の中にlin型が含まれていた場合は、型付けエラー
            let (elin, _) = env.pop(depth);
            for (k, v) in elin.unwrap().iter() {
                if v.is_some() {
                    return Err(
                        format!("関数定義内でlin型の変数\{k}\"を消費していない").into()
                    );
                }
            }

            // lin用の型環境を復元
            if let Some(ep) = env_prev {
                env.env_lin = ep;
            }   

            // 関数の型を生成
            parser::PrimType::Arrow(Box::new(e.ty.clone()), Box::new(t))

        }
    };

    // 修飾子付き型を返す
    Ok(parser::TypeExpr{
        qual: expr.qual,
        prim: p
    })
}